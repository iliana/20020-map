use crate::Team;
use anyhow::Result;
use askama::Template;
use contrast::contrast;
use tiny_skia::Pixmap;

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

pub(crate) fn render(team: &Team) -> Result<Vec<u8>> {
    let black: f64 = contrast(team.color.into(), BLACK.into());
    let white: f64 = contrast(team.color.into(), WHITE.into());
    let svg = Label {
        team,
        contrast_color: if white > black { WHITE } else { BLACK },
    }
    .render()?;

    lazy_static::lazy_static! {
        static ref OPTIONS: usvg::Options = usvg::Options {
            fontdb: {
                let mut db = usvg::fontdb::Database::new();
                db.load_font_data(include_bytes!("../data/Roboto-Bold.ttf").to_vec());
                db
            },
            ..Default::default()
        };
    }

    let tree = usvg::Tree::from_str(&svg, &OPTIONS)?;
    let mut pixmap = Pixmap::new(160, 360).unwrap();
    resvg::render(&tree, usvg::FitTo::Original, pixmap.as_mut());
    Ok(oxipng::optimize_from_memory(
        &pixmap.encode_png()?,
        &Default::default(),
    )?)
}

#[derive(Debug, Template)]
#[template(path = "label.svg", escape = "xml")]
struct Label<'a> {
    team: &'a Team,
    contrast_color: [u8; 3],
}

mod filters {
    pub(crate) fn css_color(color: &[u8; 3]) -> askama::Result<String> {
        Ok(format!("#{}", hex::encode(color)))
    }
}
