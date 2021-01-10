use crate::geometry::{Cartographic, LatLonBox};
use askama::Template;
use uom::si::f64::Angle;

#[derive(Debug, Template)]
#[template(path = "20020.kml", escape = "xml")]
pub(crate) struct Output {
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) color: [u8; 3],
    pub(crate) field: Vec<Cartographic>,
    pub(crate) line: Vec<Cartographic>,
    pub(crate) label_box: LatLonBox,
    pub(crate) label_heading: Angle,
    pub(crate) label_region_box: LatLonBox,
}

mod filters {
    use crate::geometry::Cartographic;
    use uom::si::angle::degree;
    use uom::si::f64::Angle;

    pub(super) fn degrees(value: &Angle) -> askama::Result<String> {
        Ok(format!("{}", value.get::<degree>()))
    }

    pub(super) fn kml_color(color: &[u8; 3], alpha: &f64) -> askama::Result<String> {
        Ok(hex::encode([
            (alpha * 255.0) as u8,
            color[2],
            color[1],
            color[0],
        ]))
    }

    pub(super) fn kml_coord(coord: &Cartographic) -> askama::Result<String> {
        Ok(format!(
            "{},{}",
            coord.longitude.get::<degree>(),
            coord.latitude.get::<degree>(),
        ))
    }
}
