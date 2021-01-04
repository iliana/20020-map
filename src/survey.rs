use crate::geometry::{Cartographic, Mercator, MercatorLine};
use crate::Result;
use derive_more::{Add, Sum};
use itertools::Itertools;
use regex::Regex;
use std::str::FromStr;
use uom::si::angle::degree;
use uom::si::f64::Angle;

#[derive(Debug)]
pub(crate) struct Survey {
    pub(crate) field: Mercator,
    pub(crate) line: MercatorLine,
}

pub(crate) fn parse_coordinates(input: &str) -> Result<Vec<Mercator>> {
    lazy_static::lazy_static! {
        static ref COORDINATE_RE: Regex = Regex::new(r"(?x)
            <coordinates>\s*
            ([0-9\.-]+)
            ,([0-9\.-]+)
            (?:,[0-9\.-]+)
            \s*</coordinates>
        ").unwrap();
    }

    COORDINATE_RE
        .captures_iter(input)
        .map(|captures| {
            Ok(Cartographic {
                longitude: Angle::new::<degree>(f64::from_str(&captures[1])?),
                latitude: Angle::new::<degree>(f64::from_str(&captures[2])?),
            }
            .into())
        })
        .collect()
}

/// Calculates the field location as the average of hash mark survey markers, and the heading as
/// the average heading in both the parallel and (reciprocal of) perpendicular lines between the
/// survey markers. Good when we only have the ten markers to work with.
pub(crate) fn hash_mark_survey(input: &[Mercator]) -> Survey {
    let field = input.iter().copied().sum::<Mercator>() / input.len() as f64;
    let mut lines: Vec<_> = input
        .iter()
        .copied()
        .tuple_combinations()
        .map(|(a, b)| (a.distance(b), a.slope(b)))
        .collect();
    lines.sort_unstable_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    let slope = lines
        .iter()
        .copied()
        .take(5)
        .map(|(_, slope)| -slope.recip())
        .chain(
            lines
                .iter()
                .copied()
                .skip(5)
                .take(8)
                .map(|(_, slope)| slope),
        )
        .sum::<f64>()
        / 13.0;
    Survey {
        field,
        line: MercatorLine::from_slope(slope, field),
    }
}

/// Calculates the field location as the average of the first 10 survey markers, expected to be the
/// hash marks, and the heading as a linear regression line across all survey markers. Good for
/// when we've surveyed a field but added additional markers based on other details in the survey.
pub(crate) fn linear_regression_survey(input: &[Mercator]) -> Survey {
    let field = input.iter().copied().take(10).sum::<Mercator>() / 10.0;

    #[derive(Clone, Copy, Add, Sum)]
    struct Part {
        x: f64,
        y: f64,
        xy: f64,
        x2: f64,
    }
    let sum = input
        .iter()
        .copied()
        .map(|point| Part {
            x: point.x,
            y: point.y,
            xy: point.x * point.y,
            x2: point.x.powi(2),
        })
        .sum::<Part>();
    let n = input.len() as f64;
    let slope = (n * sum.xy - sum.x * sum.y) / (n * sum.x2 - sum.x.powi(2));

    Survey {
        field,
        line: MercatorLine::from_slope(slope, field),
    }
}
