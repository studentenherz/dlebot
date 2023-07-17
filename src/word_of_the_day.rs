use std::io;
use std::path::Path;

use rand::Rng;
use usvg::{fontdb, TreeParsing, TreeTextToPath};

const BG_COLORS_LENGTH: usize = 8;

const LIGHT_BG_COLORS: [&str; BG_COLORS_LENGTH] = [
    "#f9b5b5", "#9cf2dc", "#9becf2", "#6ab6e9", "#cabff9", "#dbb3ef", "#f2cdea", "#f99daf",
];

const DARK_BG_COLORS: [&str; BG_COLORS_LENGTH] = [
    "#602323", "#187f65", "#1a5055", "#1a4563", "#312569", "#5b3171", "#633057", "#652e39",
];

pub fn get_image() -> io::Result<()> {
    let mut rng = rand::thread_rng();

    let bg_index = rng.gen_range(0..BG_COLORS_LENGTH);
    let dark_theme = rng.gen_bool(0.5);

    let bg_color = if dark_theme {
        DARK_BG_COLORS[bg_index]
    } else {
        LIGHT_BG_COLORS[bg_index]
    };

    let svg_str = format!(
        include_str!("templates/template.svg"),
        bg_color = bg_color,
        lemma = "lemma",
        etymology = "etymology",
        date = "15/7/2023",
        font_color = if dark_theme { "#ffffff" } else { "#000000" }
    );

    // resvg::Tree own all the required data and does not require
    // the input file, usvg::Tree or anything else.
    let tree = {
        let opt = usvg::Options {
            font_family: "Georgia".to_string(),
            ..Default::default()
        };

        let mut font_db = fontdb::Database::new();
        font_db.load_fonts_dir(Path::new("fonts"));

        let mut tree = usvg::Tree::from_str(&svg_str, &opt).unwrap();
        tree.convert_text(&font_db);
        resvg::Tree::from_usvg(&tree)
    };

    let pixmap_size = tree.size.to_int_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    tree.render(usvg::Transform::default(), &mut pixmap.as_mut());
    pixmap.save_png(Path::new(&format!(
        "colors/dark={}; bg_color={}.png",
        dark_theme, bg_color
    )))?;

    Ok(())
}
