use crate::geometry::{Cartographic, Mercator, MercatorLine};
use crate::Result;
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
