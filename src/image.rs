use std::path::Path;

use ::teloxide::{prelude::*, types::InputFile};
use chrono::{offset::Local, Datelike};
use rand::Rng;
use usvg::{fontdb, TreeParsing, TreeTextToPath};

use crate::{
    database::DleModel,
    utils::{base64_encode, split_by_whitespace},
    DLEBot,
};

const BG_COLORS_LENGTH: usize = 8;
const LIGHT_BG_COLORS: [&str; BG_COLORS_LENGTH] = [
    "#f9b5b5", "#9cf2dc", "#9becf2", "#6ab6e9", "#cabff9", "#dbb3ef", "#f2cdea", "#f99daf",
];
const DARK_BG_COLORS: [&str; BG_COLORS_LENGTH] = [
    "#602323", "#187f65", "#1a5055", "#1a4563", "#312569", "#5b3171", "#633057", "#652e39",
];

const MAX_CHARACTERS_IN_LINE: usize = 76;
const INTERLINE_SPACING: f64 = 1.25;
const FONT_SIZE: f64 = 4.23333;

fn get_image(lemma: &str, etymology: &str) -> Result<Vec<u8>, png::EncodingError> {
    let mut rng = rand::thread_rng();

    let bg_index = rng.gen_range(0..BG_COLORS_LENGTH);
    let dark_theme = rng.gen_bool(0.5);

    let bg_color = if dark_theme {
        DARK_BG_COLORS[bg_index]
    } else {
        LIGHT_BG_COLORS[bg_index]
    };

    let date = Local::now();
    let date = format!("{}/{}/{}", date.day(), date.month(), date.year());

    let svg_str = format!(
        include_str!("templates/template.svg"),
        bg_color = bg_color,
        lemma = lemma,
        etymology = etymology,
        date = date,
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

    pixmap.encode_png()
}

pub async fn send_image(word: DleModel, bot: DLEBot, chat_id: ChatId) -> ResponseResult<()> {
    let mut split = word.definition.trim_start().split('\n');
    let lemma = split.next().unwrap().trim();
    let mut etymology = split.next().unwrap().trim();
    if etymology.is_empty() {
        etymology = split.next().unwrap().trim();
    }

    let dy = INTERLINE_SPACING * FONT_SIZE;
    let etymology_lines: Vec<String> = split_by_whitespace(etymology, MAX_CHARACTERS_IN_LINE)
        .iter()
        .map(|&line| {
            line.replace("<i>", r#"<tspan style="font-style:italic">"#)
                .replace("<em>", r#"<tspan style="font-style:italic">"#)
                .replace("<b>", r#"<tspan style="font-style:bold">"#)
                .replace("<strong>", r#"<tspan style="font-style:bold">"#)
                .replace("<u>", r#"<tspan text-decoration="underline">"#)
                .replace("<ins>", r#"<tspan text-decoration="underline">"#)
                .replace("</i>", r#"</tspan>"#)
                .replace("</em>", r#"</tspan>"#)
                .replace("</b>", r#"</tspan>"#)
                .replace("</strong>", r#"</tspan>"#)
                .replace("</u>", r#"</tspan>"#)
                .replace("</ins>", r#"</tspan>"#)
        })
        .collect();

    let mut etymology = String::new();
    for (i, line) in etymology_lines.iter().enumerate() {
        etymology += &format!(r#"<tspan x="10" dy="{}">{}</tspan>"#, i as f64 * dy, line);
    }

    let definition = word.definition.replacen(
        lemma,
        &format!(
            r#"<a href="https://t.me/{}?start={}">{}</a>"#,
            bot.get_me().await.unwrap().username(),
            base64_encode(word.lemma.to_string()),
            lemma
        ),
        1,
    );

    if let Ok(image) = get_image(lemma, &etymology) {
        bot.send_photo(chat_id, InputFile::memory(image))
            .caption(definition)
            .await?;
    }

    Ok(())
}
