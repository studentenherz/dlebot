use std::path::Path;

use ::teloxide::{prelude::*, types::InputFile};
use chrono::{offset::Local, Datelike};
use rand::Rng;
use regex::Regex;
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

const MAX_CHARACTERS_IN_LINE: usize = 80;
const INTERLINE_SPACING: f64 = 1.25;
const FONT_SCALE: f64 = 0.9;
const FONT_SIZE_NORMAL: f64 = 5.0 * FONT_SCALE;
const FONT_SIZE_BIG: f64 = 20.0 * FONT_SCALE;

fn get_image(lemma: &str, etymology: &str, channel: &str) -> Result<Vec<u8>, png::EncodingError> {
    let mut rng = rand::thread_rng();

    let bg_index = rng.gen_range(0..BG_COLORS_LENGTH);
    let dark_theme = rng.gen_bool(0.5);

    let bg_color = if dark_theme {
        DARK_BG_COLORS[bg_index]
    } else {
        LIGHT_BG_COLORS[bg_index]
    };

    let date = if channel.is_empty() {
        "".to_string()
    } else {
        let date = Local::now();
        format!("{}/{}/{}", date.day(), date.month(), date.year())
    };

    let svg_str = format!(
        include_str!("templates/template.svg"),
        bg_color = bg_color,
        lemma = lemma,
        etymology = etymology,
        date = date,
        font_color = if dark_theme { "#ffffff" } else { "#000000" },
        channel = channel,
        font_size_normal = FONT_SIZE_NORMAL,
        font_size_big = FONT_SIZE_BIG
    );

    // resvg::Tree own all the required data and does not require
    // the input file, usvg::Tree or anything else.
    let tree = {
        let opt = usvg::Options {
            font_family: "Tinos".to_string(),
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

trait ConvertHtmlTagsToSvg {
    fn convert_html_tags_to_svg(&self) -> String;
}

impl ConvertHtmlTagsToSvg for &str {
    fn convert_html_tags_to_svg(&self) -> String {
        self.replace("<i>", r#"<tspan style="font-style:italic">"#)
            .replace("<em>", r#"<tspan style="font-style:italic">"#)
            .replace("<b>", r#"<tspan style="font-weight:bold">"#)
            .replace("<strong>", r#"<tspan style="font-weight:bold">"#)
            .replace("<u>", r#"<tspan text-decoration="underline">"#)
            .replace("<ins>", r#"<tspan text-decoration="underline">"#)
            .replace("</i>", r#"</tspan>"#)
            .replace("</em>", r#"</tspan>"#)
            .replace("</b>", r#"</tspan>"#)
            .replace("</strong>", r#"</tspan>"#)
            .replace("</u>", r#"</tspan>"#)
            .replace("</ins>", r#"</tspan>"#)
    }
}

fn fix_tags(input: &mut [String]) {
    let re = Regex::new(r#"(?P<open><tspan\s.+?>)|(?P<close></tspan>)"#).unwrap();
    let mut carry = String::new();

    for line in input.iter_mut() {
        *line = format!("{}{}", carry, line);
        let mut opening_tags: Vec<String> = vec![];
        let mut opening_tags_count = 0;
        let mut closing_tags_count = 0;
        for capture in re.captures_iter(line) {
            if capture.name("open").is_some() {
                opening_tags_count += 1;
                opening_tags.push(capture[0].into());
            } else if capture.name("close").is_some() {
                closing_tags_count += 1;
            }
        }
        if opening_tags_count > closing_tags_count {
            let diff = opening_tags_count - closing_tags_count;
            *line = format!("{}{}", line, "</tspan>".repeat(diff));
            carry = opening_tags[closing_tags_count..].join("");
        } else {
            carry.clear();
        }
    }
}

pub async fn send_image(
    word: DleModel,
    bot: DLEBot,
    chat_id: ChatId,
    pdd: bool,
) -> ResponseResult<()> {
    let mut split = word.definition.trim_start().split('\n');
    let lemma = split.next().unwrap().trim().convert_html_tags_to_svg();
    let mut etymology = split.next().unwrap().trim();
    if etymology.is_empty() {
        etymology = split.next().unwrap().trim();
    }

    let dy = INTERLINE_SPACING * FONT_SIZE_NORMAL;
    let mut etymology_lines: Vec<String> = split_by_whitespace(etymology, MAX_CHARACTERS_IN_LINE)
        .iter()
        .map(|&line| line.convert_html_tags_to_svg())
        .collect();
    fix_tags(&mut etymology_lines);

    let mut etymology = String::new();
    for (i, line) in etymology_lines.iter().enumerate() {
        etymology += &format!(r#"<tspan x="10" dy="{}">{}</tspan>"#, i as f64 * dy, line);
    }

    let definition = word.definition.replacen(
        &lemma,
        &format!(
            r#"<a href="https://t.me/{}?start={}">{}</a>"#,
            bot.get_me().await.unwrap().username(),
            base64_encode(word.lemma.to_string()),
            lemma
        ),
        1,
    );

    let mut channel = String::new();
    if let Ok(chat) = bot.get_chat(chat_id).await {
        log::info!("Chat: {:?}", chat);
        if chat.is_channel() {
            if let Some(username) = chat.username() {
                log::info!("Channel username: {}", username);
                channel = format!(r#"<tspan fill-opacity="0.7">t.me/</tspan>{} "#, username);
                log::info!("Channel: {}", channel);
            }
        }
    }

    if let Ok(image) = get_image(&lemma, &etymology, &channel) {
        bot.send_photo(chat_id, InputFile::memory(image))
            .caption(format!(
                "{} {}",
                if pdd { "📖 #PalabraDelDía |" } else { "" },
                definition.trim()
            ))
            .await?;
    }

    Ok(())
}
