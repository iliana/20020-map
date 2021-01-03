use crate::geometry::{Cartographic, MercatorSegment};
use crate::survey::Survey;
use crate::{Error, Result};
use itertools::Itertools;
use uom::si::angle::degree;
use uom::si::f64::Angle;

#[derive(Debug)]
pub(crate) struct Boundary(Vec<MercatorSegment>);

impl Boundary {
    pub(crate) fn load(input: &str) -> Result<Boundary> {
        input
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| {
                line.splitn(3, ',')
                    .take(2)
                    .map(|s| s.parse().map(Angle::new::<degree>).map_err(Error::from))
                    .collect_tuple()
                    .ok_or(Error::BoundaryInsufficient)
                    .and_then(|(longitude, latitude)| {
                        Ok(Cartographic {
                            longitude: longitude?,
                            latitude: latitude?,
                        }
                        .into())
                    })
            })
            .tuple_windows()
            .map(|(a, b)| Ok(MercatorSegment { a: a?, b: b? }))
            .collect::<Result<_>>()
            .map(Boundary)
    }

    pub(crate) fn limit(&self, survey: &Survey) -> Option<MercatorSegment> {
        let (west, east) = self
            .0
            .iter()
            .filter_map(|segment| {
                segment
                    .intersection(survey.line)
                    .map(|i| (i, survey.field.distance(i)))
            })
            .partition::<Vec<_>, _>(|(intersection, _)| intersection.x < survey.field.x);
        let (a, b) = vec![west, east]
            .into_iter()
            .filter_map(|v| {
                v.into_iter()
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .map(|(i, _)| i)
            })
            .collect_tuple()?;
        Some(MercatorSegment { a, b })
    }
}
