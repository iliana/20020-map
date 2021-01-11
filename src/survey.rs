use crate::geometry::{Cartographic, Mercator, MercatorLine, MercatorSegment};
use derive_more::{Add, Sum};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::str::FromStr;
use uom::si::angle::degree;
use uom::si::f64::Angle;

#[derive(Debug)]
pub(crate) struct Survey {
    pub(crate) field: Mercator,
    pub(crate) line: MercatorLine,
}

const COORD_START: &str = r"<coordinates>";
const COORD_END: &str = r"</coordinates>";
const POINT: &str = r"([0-9\.-]+),([0-9\.-]+)(?:,[0-9\.-]+)";

fn point(captures: &Captures<'_>, i: usize) -> Mercator {
    Cartographic {
        longitude: Angle::new::<degree>(f64::from_str(&captures[i]).unwrap()),
        latitude: Angle::new::<degree>(f64::from_str(&captures[i + 1]).unwrap()),
    }
    .into()
}

fn placemarks(kml: &str) -> impl Iterator<Item = Mercator> + '_ {
    lazy_static! {
        static ref MARK_RE: Regex = Regex::new(&format!(
            "{}{s}{}{s}{}",
            COORD_START,
            POINT,
            COORD_END,
            s = r"\s*"
        ))
        .unwrap();
    }

    MARK_RE
        .captures_iter(kml)
        .map(|captures| point(&captures, 1))
}

fn lines(kml: &str) -> impl Iterator<Item = MercatorSegment> + '_ {
    lazy_static! {
        static ref LINE_RE: Regex = Regex::new(&format!(
            "{}{s}{}{s}{}{s}{}",
            COORD_START,
            POINT,
            POINT,
            COORD_END,
            s = r"\s*"
        ))
        .unwrap();
    }

    LINE_RE.captures_iter(kml).map(|captures| MercatorSegment {
        a: point(&captures, 1),
        b: point(&captures, 3),
    })
}

fn linear_regression(points: impl Iterator<Item = Mercator>) -> f64 {
    #[derive(Clone, Copy, Add, Sum)]
    struct Part {
        x: f64,
        y: f64,
        xy: f64,
        x2: f64,
    }

    let input = points.collect::<Vec<_>>();
    let n = input.len() as f64;
    let sum = input
        .into_iter()
        .map(|point| Part {
            x: point.x,
            y: point.y,
            xy: point.x * point.y,
            x2: point.x.powi(2),
        })
        .sum::<Part>();
    (n * sum.xy - sum.x * sum.y) / (n * sum.x2 - sum.x.powi(2))
}

/// Expects a KML file of 3 lines and any number of placemarks. The first line is expected to be
/// the 50 yard line. The next 2 lines are expected to be the sidelines.
///
/// Calculates the field location as the center of the 50 yard line's intersection with the
/// sidelines. Calculates the heading as the linear regression of the sideline points and any
/// additional placemarks.
pub(crate) fn sidelines_and_50(kml: &str) -> Survey {
    let mut iter = lines(kml);
    let fifty = iter.next().unwrap().as_line();
    let sidelines = iter.take(2).collect::<Vec<_>>();

    let field = MercatorSegment::from(
        sidelines
            .iter()
            .copied()
            .filter_map(|l| fifty.intersection(l.as_line()))
            .collect_tuple::<(_, _)>()
            .unwrap(),
    )
    .midpoint();
    let slope = linear_regression(
        sidelines
            .into_iter()
            .flat_map(|segment| vec![segment.a, segment.b])
            .chain(placemarks(kml)),
    );

    Survey {
        field,
        line: MercatorLine::from_slope(slope, field),
    }
}

/// Expects a KML file of 10 or more placemarks. The first 10 placemarks are expected to be along
/// the hashmarks at the 10, 30, and 50 yard lines.
///
/// Calculates the field location as the average of the first 10 placemarks. If there are only 10
/// placemarks, the heading is the average of both the parallel and perpendicular lines between the
/// placemarks. If there are more than 10, the heading is taken as a linear regression of all
/// placemarks.
pub(crate) fn hash_mark(kml: &str) -> Survey {
    let input = placemarks(kml).collect::<Vec<_>>();
    let field = input.iter().copied().take(10).sum::<Mercator>() / 10.0;

    let slope = if input.len() > 10 {
        linear_regression(input.into_iter())
    } else {
        let mut lines: Vec<_> = input
            .into_iter()
            .tuple_combinations()
            .map(|(a, b)| (a.distance(b), a.slope(b)))
            .collect();
        lines.sort_unstable_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

        let mut parallel = lines.split_off(5);
        parallel.truncate(8);
        let perpendicular = lines;

        (parallel.into_iter().map(|(_, s)| s).sum::<f64>()
            + perpendicular
                .into_iter()
                .map(|(_, s)| -s.recip())
                .sum::<f64>())
            / 13.0
    };

    Survey {
        field,
        line: MercatorLine::from_slope(slope, field),
    }
}
