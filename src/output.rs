use crate::geometry::Cartographic;
use askama::Template;
use std::fmt;
use std::io;

#[derive(Debug, Template)]
#[template(path = "20020.kml", escape = "xml")]
pub(crate) struct Output {
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) color: [u8; 3],
    pub(crate) line: Vec<Cartographic>,
}

mod filters {
    use crate::geometry::Cartographic;
    use uom::si::angle::degree;

    pub(crate) fn kml_color(color: &[u8; 3]) -> askama::Result<String> {
        Ok(hex::encode([0xff, color[2], color[1], color[0]]))
    }

    pub(crate) fn kml_coord(coord: &Cartographic) -> askama::Result<String> {
        Ok(format!(
            "{},{}",
            coord.longitude.get::<degree>(),
            coord.latitude.get::<degree>(),
        ))
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

pub(crate) struct Adapter<T: io::Write>(pub(crate) T);

impl<T: io::Write> fmt::Write for Adapter<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.0.write_all(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}
