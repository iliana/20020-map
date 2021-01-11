use crate::ord::OrdF64;
use crate::survey::Survey;
use derive_more::{Add, Div, Sub, Sum};
use itertools::Itertools;
use std::f64::consts::FRAC_PI_2;
use uom::si::angle::{degree, radian};
use uom::si::f64::{Angle, Length};
use uom::si::length::meter;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Cartographic {
    pub(crate) longitude: Angle,
    pub(crate) latitude: Angle,
}

impl Cartographic {
    // port of https://github.com/georust/geo/blob/geo-0.16.0/geo/src/algorithm/haversine_destination.rs#L33-L48
    // to reduce deps + radians-to-degrees conversions
    pub(crate) fn destination(self, heading: Angle, distance: Length) -> Cartographic {
        let radius = Angle::from(distance / Length::new::<meter>(6_371_008.8));
        let latitude = (self.latitude.sin() * radius.cos()
            + self.latitude.cos() * radius.sin() * heading.cos())
        .asin();
        Cartographic {
            longitude: { heading.sin() * radius.sin() * self.latitude.cos() }
                .atan2(radius.cos() - self.latitude.sin() * latitude.sin())
                + self.longitude,
            latitude,
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

pub(crate) type LatLonQuad = [Cartographic; 4];

#[derive(Debug, Clone, Copy)]
pub(crate) struct LatLonBox {
    pub(crate) north: Angle,
    pub(crate) south: Angle,
    pub(crate) east: Angle,
    pub(crate) west: Angle,
}

impl LatLonBox {
    pub(crate) fn new(center: Cartographic, height: Length, width: Length) -> LatLonBox {
        LatLonBox {
            north: center
                .destination(Angle::new::<radian>(0.0), height / 2.0)
                .latitude,
            south: center
                .destination(Angle::new::<radian>(0.0), -height / 2.0)
                .latitude,
            east: center
                .destination(Angle::new::<radian>(FRAC_PI_2), width / 2.0)
                .longitude,
            west: center
                .destination(Angle::new::<radian>(FRAC_PI_2), -width / 2.0)
                .longitude,
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Clone, Copy, Add, Div, Sub, Sum)]
pub(crate) struct Mercator {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Mercator {
    pub(crate) fn distance(self, other: Mercator) -> f64 {
        let diff = other - self;
        (diff.x.powi(2) + diff.y.powi(2)).sqrt()
    }

    pub(crate) fn slope(self, other: Mercator) -> f64 {
        let diff = other - self;
        diff.y / diff.x
    }
}

impl From<Cartographic> for Mercator {
    fn from(point: Cartographic) -> Mercator {
        Mercator {
            x: point.longitude.get::<radian>(),
            y: point.latitude.get::<radian>().tan().asinh(),
        }
    }
}

impl From<Mercator> for Cartographic {
    fn from(point: Mercator) -> Cartographic {
        Cartographic {
            longitude: Angle::new::<radian>(point.x),
            latitude: Angle::new::<radian>(point.y.sinh().atan()),
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Clone, Copy)]
pub(crate) struct MercatorSegment {
    pub(crate) a: Mercator,
    pub(crate) b: Mercator,
}

impl MercatorSegment {
    pub(crate) fn as_line(self) -> MercatorLine {
        MercatorLine::new(self.a, self.b)
    }

    pub(crate) fn intersection(self, line: MercatorLine) -> Option<Mercator> {
        let intersection = self.as_line().intersection(line)?;
        if self.a.x.min(self.b.x) <= intersection.x && intersection.x <= self.a.x.max(self.b.x) {
            Some(intersection)
        } else {
            None
        }
    }

    pub(crate) fn midpoint(self) -> Mercator {
        (self.a + self.b) / 2.0
    }

    pub(crate) fn plot(self) -> Vec<Cartographic> {
        let line = self.as_line();
        let mut l = Vec::new();
        let d_x = 0.0005 / (line.slope.powi(2) + 1.0).sqrt();
        let mut x = self.a.x.min(self.b.x);
        let end = self.a.x.max(self.b.x);
        while x < end {
            l.push(
                Mercator {
                    x,
                    y: line.slope * x + line.y_intercept,
                }
                .into(),
            );
            x += d_x;
        }
        l.push(
            Mercator {
                x: end,
                y: line.slope * end + line.y_intercept,
            }
            .into(),
        );
        l
    }
}

impl From<(Mercator, Mercator)> for MercatorSegment {
    fn from(x: (Mercator, Mercator)) -> MercatorSegment {
        MercatorSegment { a: x.0, b: x.1 }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Clone, Copy)]
pub(crate) struct MercatorLine {
    slope: f64,
    y_intercept: f64,
}

impl MercatorLine {
    pub(crate) fn new(start: Mercator, end: Mercator) -> MercatorLine {
        MercatorLine::from_slope(start.slope(end), start)
    }

    pub(crate) fn from_slope(slope: f64, point: Mercator) -> MercatorLine {
        MercatorLine {
            slope,
            y_intercept: point.y - slope * point.x,
        }
    }

    pub(crate) fn heading(self) -> Angle {
        Angle::new::<radian>(FRAC_PI_2 - self.slope.atan())
    }

    pub(crate) fn intersection(self, other: MercatorLine) -> Option<Mercator> {
        if (self.slope - other.slope).abs() < (f64::EPSILON * self.slope.max(other.slope)) {
            return None;
        }
        let x = (other.y_intercept - self.y_intercept) / (self.slope - other.slope);
        Some(Mercator {
            x,
            y: self.slope * x + self.y_intercept,
        })
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug)]
pub(crate) struct Boundary(Vec<MercatorSegment>);

impl Boundary {
    pub(crate) fn load(input: &str) -> Boundary {
        Boundary(
            input
                .lines()
                .filter(|line| !line.starts_with('#'))
                .map(|line| {
                    let (longitude, latitude) = line
                        .splitn(3, ',')
                        .take(2)
                        .map(|s| Angle::new::<degree>(s.parse().unwrap()))
                        .collect_tuple()
                        .expect("insufficient data in boundary data line");
                    Cartographic {
                        longitude,
                        latitude,
                    }
                    .into()
                })
                .tuple_windows()
                .map(|(a, b)| MercatorSegment { a, b })
                .collect(),
        )
    }

    pub(crate) fn limit(&self, survey: &Survey) -> Option<MercatorSegment> {
        let (west, east) = self
            .0
            .iter()
            .filter_map(|segment| {
                segment
                    .intersection(survey.line)
                    .map(|i| (i, OrdF64(survey.field.distance(i))))
            })
            .partition::<Vec<_>, _>(|(intersection, _)| intersection.x < survey.field.x);
        let (a, b) = vec![west, east]
            .into_iter()
            .filter_map(|v| v.into_iter().min_by_key(|(_, d)| *d).map(|(i, _)| i))
            .collect_tuple()?;
        Some(MercatorSegment { a, b })
    }
}
