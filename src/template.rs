use crate::geometry::{Cartographic, LatLonBox, LatLonQuad};
use crate::Team;
use askama::Template;
use uom::si::f64::Angle;

#[derive(Debug, Template)]
#[template(path = "20020.kml", escape = "xml")]
pub(crate) struct Output<'a> {
    pub(crate) kmz: bool,
    pub(crate) fields: &'a [Field],
}

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) color: [u8; 3],
    pub(crate) field: Vec<Cartographic>,
    pub(crate) field_quads: Vec<LatLonQuad>,
    pub(crate) line: Vec<Cartographic>,
    pub(crate) label_box: LatLonBox,
    pub(crate) label_heading: Angle,
    pub(crate) label_region_box: LatLonBox,
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Template)]
#[template(path = "label.svg", escape = "xml")]
pub(crate) struct Label<'a> {
    pub(crate) team: &'a Team,
    pub(crate) contrast_color: [u8; 3],
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

mod filters {
    use crate::geometry::{Cartographic, LatLonQuad};
    use crate::ord::OrdF64;
    use askama::Result;
    use paste::paste;
    use uom::si::angle::degree;
    use uom::si::f64::Angle;

    pub(crate) fn css_color(color: &[u8; 3]) -> Result<String> {
        Ok(hex::encode(color))
    }

    pub(super) fn degrees(value: &Angle) -> Result<String> {
        Ok(format!("{}", value.get::<degree>()))
    }

    pub(super) fn kml_color(color: &[u8; 3], alpha: &f64) -> Result<String> {
        Ok(hex::encode([
            (255.0 * alpha) as u8,
            color[2],
            color[1],
            color[0],
        ]))
    }

    pub(super) fn kml_coord(coord: &Cartographic) -> Result<String> {
        Ok(format!(
            "{},{}",
            coord.longitude.get::<degree>(),
            coord.latitude.get::<degree>(),
        ))
    }

    macro_rules! limit {
        ($dir:ident, $value:ident, $f:ident) => {
            paste! {
                pub(super) fn [<limit_ $dir>](quad: &LatLonQuad) -> Result<String> {
                    Ok(format!(
                        "{}",
                        quad.iter()
                            .copied()
                            .map(|c| OrdF64(c.$value.get::<degree>()))
                            .$f()
                            .unwrap()
                    ))
                }
            }
        };
    }
    limit!(north, latitude, max);
    limit!(south, latitude, min);
    limit!(east, longitude, max);
    limit!(west, longitude, min);
}
