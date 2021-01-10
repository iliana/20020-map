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
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Cursor, ErrorKind};
use std::path::Path;
use std::str::FromStr;
use uom::si::f64::Length;
use uom::si::length::foot;
use zip::ZipWriter;

lazy_static::lazy_static! {
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
    for line in fs::read_to_string(root().join("data").join("teams.csv"))?
        .lines()
        .skip(1)
    {
        let team = Team::from_str(line)?;
        let coordinates = survey::parse_coordinates(&match fs::read_to_string(
            root().join("survey").join(&team.name).with_extension("kml"),
        ) {
            Ok(x) => x,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        })?;

        let survey = if coordinates.len() == 10 {
            survey::hash_mark_survey(&coordinates)
        } else {
            survey::linear_regression_survey(&coordinates)
        };
        let field = Cartographic::from(survey.field);
        let line = boundary.limit(&survey).ok_or(Error::BoundaryLimit)?.plot();

        fields.push(Field {
            color: team.color,
            line,
            label: label::render(&team)?,
            label_box: LatLonBox::new(field, *LABEL_HEIGHT, *LABEL_WIDTH),
            label_heading: survey.line.heading(),
            label_region_box: LatLonBox::new(field, *LABEL_DIAGONAL, *LABEL_DIAGONAL),
            name: team.name,
        });
    }

    let site_dir = root().join("site");
    let files_dir = site_dir.join("files");
    fs::create_dir_all(&files_dir)?;

    let mut zip = ZipWriter::new(File::create(site_dir.join("20020.kmz"))?);

    for field in &fields {
        let filename = format!("{}.png", field.name);
        fs::write(files_dir.join(&filename), &field.label)?;
        zip.start_file(&format!("files/{}", filename), Default::default())?;
        io::copy(&mut Cursor::new(&field.label), &mut zip)?;
    }

    let kml = Output { fields }.render()?;
    fs::write(site_dir.join("20020.kml"), &kml)?;
    zip.start_file("doc.kml", Default::default())?;
    io::copy(&mut Cursor::new(kml.as_bytes()), &mut zip)?;

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
