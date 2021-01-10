use derive_more::{Add, Div, Sub, Sum};
use geographiclib_rs::{DirectGeodesic, Geodesic};
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
    fn geodesic(&self, heading: Angle, distance: Length) -> Cartographic {
        let (lat, lon) = Geodesic::wgs84().direct(
            self.latitude.get::<degree>(),
            self.longitude.get::<degree>(),
            heading.get::<degree>(),
            distance.get::<meter>(),
        );
        Cartographic {
            longitude: Angle::new::<degree>(lon),
            latitude: Angle::new::<degree>(lat),
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

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
                .geodesic(Angle::new::<radian>(0.0), height / 2.0)
                .latitude,
            south: center
                .geodesic(Angle::new::<radian>(0.0), -height / 2.0)
                .latitude,
            east: center
                .geodesic(Angle::new::<radian>(FRAC_PI_2), width / 2.0)
                .longitude,
            west: center
                .geodesic(Angle::new::<radian>(FRAC_PI_2), -width / 2.0)
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
    fn as_line(self) -> MercatorLine {
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
        Angle::new::<radian>(self.slope.atan() + FRAC_PI_2)
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
