pub use geo::prelude::*;

use geo::algorithm::line_interpolate_point::LineInterpolatePoint;

pub type Coordinate = geo::Coordinate<f64>;
pub type Line = geo::Line<f64>;
pub type LineString = geo::LineString<f64>;
pub type Point = geo::Point<f64>;

pub trait CoordinateExt {
    fn bearing_from_slope(self, slope: f64) -> f64;
}

impl CoordinateExt for Coordinate {
    fn bearing_from_slope(self, slope: f64) -> f64 {
        let d_x = 0.000005 / (1.0 + slope.powi(2)).sqrt();
        let a = Point::from(self);
        let b = a + Point::new(d_x, slope * d_x);
        a.bearing(b)
    }
}

pub trait LineExt {
    fn interpolate(self) -> Interpolate;
    fn intersection(self, other: Self) -> Option<Coordinate>;
    fn roughly_contains(self, point: Coordinate) -> bool;
}

impl LineExt for Line {
    fn interpolate(self) -> Interpolate {
        let delta = self.delta();
        let length = (delta.x.powi(2) + delta.y.powi(2)).sqrt();
        let step = 0.05 / length;
        Interpolate {
            line: self,
            fraction: 0.0,
            step,
            done: false,
        }
    }

    fn intersection(self, other: Line) -> Option<Coordinate> {
        let (x1, y1) = self.start.x_y();
        let (x2, y2) = self.end.x_y();
        let (x3, y3) = other.start.x_y();
        let (x4, y4) = other.end.x_y();

        let denominator = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        if denominator.abs() < 1e-15 {
            return None;
        }

        let a = x1 * y2 - y1 * x2;
        let b = x3 * y4 - y3 * x4;
        Some(Coordinate {
            x: (a * (x3 - x4) - (x1 - x2) * b) / denominator,
            y: (a * (y3 - y4) - (y1 - y2) * b) / denominator,
        })
    }

    fn roughly_contains(self, point: Coordinate) -> bool {
        let x_min = self.start.x.min(self.end.x) - 0.000_001;
        let x_max = self.start.x.max(self.end.x) + 0.000_001;
        let y_min = self.start.y.min(self.end.y) - 0.000_001;
        let y_max = self.start.y.max(self.end.y) + 0.000_001;
        (x_min..=x_max).contains(&point.x) && (y_min..=y_max).contains(&point.y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Interpolate {
    line: Line,
    fraction: f64,
    step: f64,
    done: bool,
}

impl Iterator for Interpolate {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        if self.done {
            None
        } else {
            self.done = self.fraction > 1.0;
            let point = self.line.line_interpolate_point(self.fraction);
            self.fraction += self.step;
            point.map(Coordinate::from)
        }
    }
}
