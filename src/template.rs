use crate::geo::Interpolate;
use crate::{LatLonBox, Team};
use askama::Template;

#[derive(Debug, Template)]
#[template(path = "20020.kml", escape = "xml")]
pub(crate) struct Output<'a> {
    pub kmz: bool,
    pub fields: &'a [Field],
}

#[derive(Debug)]
pub(crate) struct Field {
    pub team: Team,
    pub field: LatLonBox,
    pub field_bearing: f64,
    pub line: Interpolate,
    pub label: LatLonBox,
    pub label_bearing: f64,
    pub label_region: LatLonBox,
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Template)]
#[template(path = "label.svg", escape = "xml")]
pub(crate) struct Label<'a> {
    pub team: &'a Team,
    pub contrast_color: &'static str,
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

mod filters {
    use askama::Result;

    pub(super) fn css_color(color: &[u8; 3]) -> Result<String> {
        Ok(hex::encode(color))
    }

    pub(super) fn kml_color(color: &[u8; 3]) -> Result<String> {
        Ok(format!("ff{}", hex::encode([color[2], color[1], color[0]])))
    }
}
