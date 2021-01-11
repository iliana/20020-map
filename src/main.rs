#![deny(rust_2018_idioms)]

mod boundary;
mod geometry;
mod label;
mod output;
mod survey;

use crate::boundary::Boundary;
use crate::geometry::{Cartographic, LatLonBox};
use crate::output::{Field, Output};
use askama::Template;
use hex::FromHex;
use std::f64::consts::FRAC_PI_2;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Cursor, ErrorKind};
use std::path::Path;
use std::str::FromStr;
use uom::si::angle::radian;
use uom::si::f64::{Angle, Length};
use uom::si::length::foot;
use zip::ZipWriter;

lazy_static::lazy_static! {
    static ref QUARTER_TURN: Angle = Angle::new::<radian>(FRAC_PI_2);
    static ref FIELD_WIDTH_HALF: Length = Length::new::<foot>(80.0);
    static ref LABEL_HEIGHT: Length = Length::new::<foot>(180_000.0);
    static ref LABEL_WIDTH: Length = Length::new::<foot>(80_000.0);
    static ref LABEL_DIAGONAL: Length = ((*LABEL_HEIGHT).powi(uom::typenum::P2::new())
                                         + (*LABEL_HEIGHT).powi(uom::typenum::P2::new())).sqrt();
}

fn main() -> anyhow::Result<()> {
    let boundary = Boundary::load(&fs::read_to_string(
        root().join("data").join("boundary.csv"),
    )?)?;

    let mut fields: Vec<Field> = Vec::new();
    let mut labels: Vec<(String, Vec<u8>)> = Vec::new();
    for line in fs::read_to_string(root().join("data").join("teams.csv"))?
        .lines()
        .skip(1)
    {
        let team = Team::from_str(line)?;
        let kml = match fs::read_to_string(
            root().join("survey").join(&team.name).with_extension("kml"),
        ) {
            Ok(x) => x,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        let survey = match team.name.as_str() {
            "Purdue" => survey::sidelines_and_50(&kml),
            _ => survey::hash_mark(&kml),
        };

        let center = Cartographic::from(survey.field);
        let heading = survey.line.heading();
        let cross = heading - *QUARTER_TURN;

        let line = boundary.limit(&survey).ok_or(Error::BoundaryLimit)?.plot();
        let mut field = Vec::with_capacity(line.len() * 2);
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

        labels.push((format!("{}.png", team.name), label::render(&team)?));
        fields.push(Field {
            name: team.name,
            color: team.color,
            field,
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
    let kml = Output { fields }.render()?;
    fs::write(site_dir.join("20020.kml"), &kml)?;
    zip.start_file("doc.kml", Default::default())?;
    io::copy(&mut Cursor::new(kml.as_bytes()), &mut zip)?;

    for (filename, label) in labels {
        fs::write(files_dir.join(&filename), &label)?;
        zip.start_file(&format!("files/{}", filename), Default::default())?;
        io::copy(&mut Cursor::new(&label), &mut zip)?;
    }

    zip.finish()?;

    Ok(())
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[derive(Debug)]
struct Team {
    name: String,
    abbr: String,
    color: [u8; 3],
}

impl FromStr for Team {
    type Err = Error;

    fn from_str(s: &str) -> Result<Team> {
        let mut iter = s.splitn(3, ',');
        let name = iter.next().ok_or(Error::TeamInsufficient)?.to_string();
        let abbr = iter.next().ok_or(Error::TeamInsufficient)?.to_string();
        let color = iter.next().ok_or(Error::TeamInsufficient)?;

        Ok(Team {
            name,
            abbr,
            color: <[u8; 3]>::from_hex(color.trim_start_matches('#'))?,
        })
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
enum Error {
    BoundaryInsufficient,
    BoundaryLimit,
    FromHex(hex::FromHexError),
    ParseFloat(std::num::ParseFloatError),
    TeamInsufficient,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BoundaryInsufficient => writeln!(f, "insufficient data in boundary data line"),
            Error::BoundaryLimit => writeln!(f, "failed to limit line to boundary"),
            Error::FromHex(source) => source.fmt(f),
            Error::ParseFloat(source) => source.fmt(f),
            Error::TeamInsufficient => writeln!(f, "insufficient data in team data line"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Error {
        Error::ParseFloat(e)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Error {
        Error::FromHex(e)
    }
}
