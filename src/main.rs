#![deny(rust_2018_idioms)]
#![allow(clippy::map_entry)] // https://github.com/rust-lang/rust-clippy/issues/1450

mod geometry;
mod image;
mod ord;
mod survey;
mod template;

use crate::geometry::{Boundary, Cartographic, LatLonBox};
use crate::template::{Field, Output};
use anyhow::Result;
use askama::Template;
use hex::FromHex;
use itertools::Itertools;
use std::collections::HashMap;
use std::f64::consts::FRAC_PI_2;
use std::fs::{self, File};
use std::io::{self, Cursor, ErrorKind};
use std::path::Path;
use uom::si::angle::radian;
use uom::si::f64::{Angle, Length};
use uom::si::length::foot;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

lazy_static::lazy_static! {
    static ref QUARTER_TURN: Angle = Angle::new::<radian>(FRAC_PI_2);
    static ref FIELD_WIDTH_HALF: Length = Length::new::<foot>(80.0);
    static ref LABEL_HEIGHT: Length = Length::new::<foot>(180_000.0);
    static ref LABEL_WIDTH: Length = Length::new::<foot>(80_000.0);
    static ref LABEL_DIAGONAL: Length = ((*LABEL_HEIGHT).powi(uom::typenum::P2::new())
                                         + (*LABEL_WIDTH).powi(uom::typenum::P2::new())).sqrt();
}

fn main() -> Result<()> {
    let boundary = Boundary::load(&fs::read_to_string(
        root().join("data").join("boundary.csv"),
    )?);

    let mut fields: Vec<Field> = Vec::new();
    let mut images: HashMap<String, Vec<u8>> = HashMap::new();
    for line in fs::read_to_string(root().join("data").join("teams.csv"))?
        .lines()
        .skip(1)
    {
        let team = Team::from_str(line).expect("insufficient data in team data line");
        let kml = match fs::read_to_string(
            root().join("survey").join(&team.name).with_extension("kml"),
        ) {
            Ok(x) => x,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        let survey = match team.name.as_str() {
            "Purdue" | "UAB" => survey::sidelines_and_50(&kml),
            _ => survey::hash_mark(&kml),
        };

        let center = Cartographic::from(survey.field);
        let heading = survey.line.heading();
        let cross = heading - *QUARTER_TURN;
        let line = boundary
            .limit(&survey)
            .expect("failed to limit line to boundary")
            .plot();

        let mut field = Vec::with_capacity(line.len() + 2);
        field.extend(
            line.iter()
                .copied()
                .map(|point| point.destination(cross, *FIELD_WIDTH_HALF)),
        );
        field.extend(
            line.iter()
                .copied()
                .rev()
                .map(|point| point.destination(cross, -*FIELD_WIDTH_HALF)),
        );

        let field_quads = line
            .iter()
            .copied()
            .map(Cartographic::from)
            .tuple_windows()
            .map(|(a, b)| {
                [
                    a.destination(cross, *FIELD_WIDTH_HALF),
                    a.destination(cross, -*FIELD_WIDTH_HALF),
                    b.destination(cross, -*FIELD_WIDTH_HALF),
                    b.destination(cross, *FIELD_WIDTH_HALF),
                ]
            })
            .collect();

        images.insert(format!("{}.png", team.name), image::label(&team)?);
        let field_filename = format!("{}.png", hex::encode(team.color));
        if !images.contains_key(&field_filename) {
            images.insert(field_filename, image::field(&team)?);
        }

        fields.push(Field {
            name: team.name,
            color: team.color,
            field,
            field_quads,
            line,
            label_box: LatLonBox::new(center, *LABEL_HEIGHT, *LABEL_WIDTH),
            label_heading: Angle::HALF_TURN - heading,
            label_region_box: LatLonBox::new(center, *LABEL_DIAGONAL, *LABEL_DIAGONAL),
        });
    }

    let site_dir = root().join("site");
    let files_dir = site_dir.join("files");
    fs::create_dir_all(&files_dir)?;

    let mut zip = ZipWriter::new(File::create(site_dir.join("20020.kmz"))?);
    fs::write(
        site_dir.join("20020.kml"),
        &Output {
            kmz: false,
            fields: &fields,
        }
        .render()?,
    )?;
    zip.start_file("doc.kml", FileOptions::default())?;
    io::copy(
        &mut Cursor::new(
            Output {
                kmz: true,
                fields: &fields,
            }
            .render()?
            .as_bytes(),
        ),
        &mut zip,
    )?;

    for (filename, image) in images {
        fs::write(files_dir.join(&filename), &image)?;
        zip.start_file(
            &format!("files/{}", filename),
            FileOptions::default().compression_method(CompressionMethod::Stored),
        )?;
        io::copy(&mut Cursor::new(&image), &mut zip)?;
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
    fn from_str(s: &str) -> Option<Team> {
        let mut iter = s.splitn(3, ',');
        let name = iter.next()?.to_string();
        let abbr = iter.next()?.to_string();
        let color = iter.next()?;

        Some(Team {
            name,
            abbr,
            color: <[u8; 3]>::from_hex(color.trim_start_matches('#')).unwrap(),
        })
    }
}
