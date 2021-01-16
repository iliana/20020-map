use crate::geo::*;
use crate::ord::OrdF64;
use derive_more::{Add, Sum};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy)]
pub struct Survey {
    pub field: Coordinate,
    pub bearing: f64,
}

impl Survey {
    pub fn from_slope(field: Coordinate, slope: f64) -> Survey {
        Survey {
            field,
            bearing: field.bearing_from_slope(slope),
        }
    }

    pub fn as_line(self) -> Line {
        Line {
            start: self.field,
            end: Point::from(self.field)
                .haversine_destination(self.bearing, 50.0)
                .into(),
        }
    }
}

pub fn default(kml: &str) -> Survey {
    if kml.contains("<LineString>") {
        sidelines_and_50(kml)
    } else {
        hash_mark(kml)
    }
}

/// Expects a KML file of 3 lines and any number of placemarks. The first line is expected to be
/// the 50 yard line. The next 2 lines are expected to be the sidelines.
///
/// Calculates the field location as the center of the 50 yard line's intersection with the
/// sidelines. Calculates the heading as the linear regression of the sideline points and any
/// additional placemarks.
fn sidelines_and_50(kml: &str) -> Survey {
    let mut lines = lines(kml);
    let fifty = lines.next().unwrap();
    let sidelines = lines.collect_tuple::<(_, _)>().unwrap();

    let endpoints = (
        fifty.intersection(sidelines.0).unwrap(),
        fifty.intersection(sidelines.1).unwrap(),
    );
    let field = (endpoints.0 + endpoints.1) / 2.0;

    let mut marks = placemarks(kml).peekable();
    let slope = if marks.peek().is_some() {
        linear_regression(
            vec![
                sidelines.0.start,
                sidelines.0.end,
                sidelines.1.start,
                sidelines.1.end,
            ]
            .into_iter()
            .chain(marks),
        )
    } else {
        (sidelines.0.slope() + sidelines.1.slope()) / 2.0
    };

    Survey::from_slope(field, slope)
}

/// Expects a KML file of 10 or more placemarks. The first 10 placemarks are expected to be along
/// the hashmarks at the 10, 30, and 50 yard lines.
///
/// Calculates the field location as the average of the first 10 placemarks. If there are only 10
/// placemarks, the heading is the average of both the parallel and perpendicular lines between the
/// placemarks. If there are more than 10, the heading is taken as a linear regression of all
/// placemarks.
fn hash_mark(kml: &str) -> Survey {
    let marks = placemarks(kml).collect::<Vec<_>>();
    let field = marks
        .iter()
        .copied()
        .take(10)
        .map(Point::from)
        .fold(Point::new(0.0, 0.0), |acc, point| acc + point)
        / 10.0;

    let slope = if marks.len() > 10 {
        if config(kml, "centerfit") {
            linear_regression(marks.into_iter().skip(10).chain(vec![field.into()]))
        } else {
            linear_regression(marks.into_iter())
        }
    } else {
        let mut lines: Vec<_> = marks
            .into_iter()
            .tuple_combinations()
            .map(|(start, end)| {
                let line = Line { start, end };
                (OrdF64(line.haversine_length()), line.slope())
            })
            .collect();
        lines.sort_by_key(|(d, _)| *d);
        lines
            .into_iter()
            .skip(5)
            .take(8)
            .map(|(_, s)| s)
            .sum::<f64>()
            / 8.0
    };

    Survey::from_slope(field.into(), slope)
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

fn linear_regression(points: impl Iterator<Item = Coordinate>) -> f64 {
    #[derive(Add, Sum)]
    struct Part {
        x: f64,
        y: f64,
        xy: f64,
        x2: f64,
        n: usize,
    }

    let sum = points
        .map(|point| Part {
            x: point.x,
            y: point.y,
            xy: point.x * point.y,
            x2: point.x.powi(2),
            n: 1,
        })
        .sum::<Part>();
    let n = sum.n as f64;
    (n * sum.xy - sum.x * sum.y) / (n * sum.x2 - sum.x.powi(2))
}

fn config(kml: &str, option: &str) -> bool {
    kml.contains(&format!("[[navarro::{}]]", option))
}

macro_rules! coord_re {
    ($fmt:expr) => {{
        let float_re = r"-?[0-9]+(?:\.[0-9]+)?";
        Regex::new(&format!(
            $fmt,
            start = r"<coordinates>",
            end = r"</coordinates>",
            point = format!(r"({0}),({0})(?:,{0})?", float_re),
            s = r"\s",
        ))
    }};
}

fn placemarks(kml: &str) -> impl Iterator<Item = Coordinate> + '_ {
    lazy_static! {
        static ref RE: Regex = coord_re!("{start}{s}*{point}{s}*{end}").unwrap();
    }

    RE.captures_iter(kml).map(|captures| Coordinate {
        x: captures[1].parse().unwrap(),
        y: captures[2].parse().unwrap(),
    })
}

fn lines(kml: &str) -> impl Iterator<Item = Line> + '_ {
    lazy_static! {
        static ref RE: Regex = coord_re!("{start}{s}*{point}{s}+{point}{s}*{end}").unwrap();
    }

    RE.captures_iter(kml).map(|captures| Line {
        start: Coordinate {
            x: captures[1].parse().unwrap(),
            y: captures[2].parse().unwrap(),
        },
        end: Coordinate {
            x: captures[3].parse().unwrap(),
            y: captures[4].parse().unwrap(),
        },
    })
}
