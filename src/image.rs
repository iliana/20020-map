use crate::template::Label;
use crate::Team;
use anyhow::Result;
use askama::Template;
use contrast::contrast;
use tiny_skia::{Color, Pixmap};

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

fn encode(pixmap: &Pixmap) -> Result<Vec<u8>> {
    Ok(oxipng::optimize_from_memory(
        &pixmap.encode_png()?,
        &Default::default(),
    )?)
}

pub(crate) fn field(team: &Team) -> Result<Vec<u8>> {
    let mut color = Color::from_rgba8(team.color[0], team.color[1], team.color[2], u8::MAX);
    color.set_alpha(0.7);
    let mut pixmap = Pixmap::new(1, 20).unwrap();
    pixmap.fill(color);
    encode(&pixmap)
}

pub(crate) fn label(team: &Team) -> Result<Vec<u8>> {
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
    encode(&pixmap)
}
