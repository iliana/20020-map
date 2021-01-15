#![deny(rust_2018_idioms)]
#![allow(clippy::map_entry)] // https://github.com/rust-lang/rust-clippy/issues/1450

mod geo;
mod image;
mod ord;
mod survey;
mod template;

use crate::geo::*;
use crate::ord::OrdF64;
use crate::survey::Survey;
use crate::template::*;
use anyhow::Result;
use askama::Template;
use hex::FromHex;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{prelude::*, BufReader, ErrorKind};
use std::path::Path;
use uom::si::f64::Length;
use uom::si::length::{foot, meter, mile};
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

lazy_static! {
    static ref LABEL_HEIGHT: Length = Length::new::<foot>(360.0) * 500.0;
    static ref LABEL_WIDTH: Length = Length::new::<foot>(160.0) * 500.0;
    static ref LABEL_DIAGONAL: Length = ((*LABEL_HEIGHT).powi(uom::typenum::P2::new())
        + (*LABEL_WIDTH).powi(uom::typenum::P2::new()))
    .sqrt();
}

fn main() -> Result<()> {
    let boundary = Boundary::load(BufReader::new(File::open(
        root().join("data").join("boundary.csv"),
    )?));

    let mut fields = Vec::new();
    let mut images = HashMap::new();

    for line in BufReader::new(File::open(root().join("data").join("teams.csv"))?)
        .lines()
        .skip(1)
    {
        let team = Team::from_str(&line?);
        let kml = match fs::read_to_string(
            root().join("survey").join(&team.name).with_extension("kml"),
        ) {
            Ok(x) => x,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        let survey = survey::default(&kml);

        let line = boundary.limit(&survey).unwrap();
        let center = (line.start + line.end) / 2.0;

        images.insert(format!("{}.png", team.name), image::label(&team)?);
        let field_filename = format!("{}.png", hex::encode(team.color));
        if !images.contains_key(&field_filename) {
            images.insert(field_filename, image::field(&team)?);
        }

        fields.push(Field {
            team,
            field: LatLonBox::new(
                center,
                Length::new::<foot>(160.0),
                Length::new::<meter>(line.haversine_length()),
            )
            .adjust_width(survey.field, Length::new::<foot>(160.0)),
            field_bearing: center.bearing_from_slope(line.slope()),
            line: line.interpolate(Length::new::<mile>(5.0)),
            label: LatLonBox::new(survey.field, *LABEL_WIDTH, *LABEL_HEIGHT),
            label_bearing: survey.bearing,
            label_region: LatLonBox::new(survey.field, *LABEL_DIAGONAL, *LABEL_DIAGONAL),
        });
    }

    let site_dir = root().join("site");
    let files_dir = site_dir.join("files");
    fs::create_dir_all(&files_dir)?;

    let mut zip = ZipWriter::new(File::create(site_dir.join("20020.kmz"))?);
    fs::write(
        site_dir.join("20020.kml"),
        Output {
            fields: &fields,
            kmz: false,
        }
        .render()?
        .as_bytes(),
    )?;
    zip.start_file("doc.kml", FileOptions::default())?;
    zip.write_all(
        Output {
            fields: &fields,
            kmz: true,
        }
        .render()?
        .as_bytes(),
    )?;

    for (filename, image) in images {
        zip.start_file(
            &format!("files/{}", filename),
            FileOptions::default().compression_method(CompressionMethod::Stored),
        )?;
        zip.write_all(&image)?;
    }

    zip.finish()?;
    Ok(())
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug)]
struct Team {
    name: String,
    abbr: String,
    color: [u8; 3],
}

impl Team {
    fn from_str(s: &str) -> Team {
        let mut iter = s.split(',');
        Team {
            name: iter.next().unwrap().to_string(),
            abbr: iter.next().unwrap().to_string(),
            color: <[u8; 3]>::from_hex(iter.next().unwrap().trim_start_matches('#')).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LatLonBox {
    north: f64,
    south: f64,
    east: f64,
    west: f64,
}

impl LatLonBox {
    fn new(center: Coordinate, width: Length, height: Length) -> LatLonBox {
        let center = Point::from(center);
        LatLonBox {
            north: center
                .haversine_destination(0.0, height.get::<meter>() / 2.0)
                .y(),
            south: center
                .haversine_destination(180.0, height.get::<meter>() / 2.0)
                .y(),
            east: center
                .haversine_destination(90.0, width.get::<meter>() / 2.0)
                .x(),
            west: center
                .haversine_destination(270.0, width.get::<meter>() / 2.0)
                .x(),
        }
    }

    fn adjust_width(self, at: Coordinate, width: Length) -> LatLonBox {
        let lon = (self.east + self.west) / 2.0;
        let angle = Point::from(at)
            .haversine_destination(90.0, width.get::<meter>() / 2.0)
            .x()
            - at.x;
        LatLonBox {
            east: lon + angle,
            west: lon - angle,
            ..self
        }
    }
}

#[derive(Debug)]
struct Boundary(LineString);

impl Boundary {
    fn load(input: impl BufRead) -> Boundary {
        Boundary(
            input
                .lines()
                .map(|line| line.unwrap())
                .filter(|line| !line.starts_with('#'))
                .map(|line| {
                    Coordinate::from(
                        line.splitn(3, ',')
                            .take(2)
                            .map(|s| s.parse().unwrap())
                            .collect_tuple::<(_, _)>()
                            .unwrap(),
                    )
                })
                .collect(),
        )
    }

    fn limit(&self, survey: &Survey) -> Option<Line> {
        let survey_line = survey.as_line();
        let (west, east) = self
            .0
            .lines()
            .filter_map(|line| {
                if let Some(i) = line.intersection(survey_line) {
                    if line.roughly_contains(i) {
                        return Some((
                            i,
                            OrdF64(Point::from(survey.field).haversine_distance(&i.into())),
                        ));
                    }
                }
                None
            })
            .partition::<Vec<_>, _>(|(intersection, _)| intersection.x < survey.field.x);
        let (start, end) = vec![west, east]
            .into_iter()
            .filter_map(|v| v.into_iter().min_by_key(|(_, d)| *d).map(|(i, _)| i))
            .collect_tuple()?;
        Some(Line { start, end })
    }
}
