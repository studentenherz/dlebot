use std::path::Path;

use ::teloxide::{prelude::*, types::InputFile};
use chrono::{offset::Local, Datelike, Duration, Timelike};
use rand::Rng;
use tokio::time::{interval_at, Duration as StdDuration, Instant};
use usvg::{fontdb, TreeParsing, TreeTextToPath};

use crate::{database::DatabaseHandler, DLEBot};

const SECONDS_IN_A_DAY: u64 = 24 * 60 * 60;
const BG_COLORS_LENGTH: usize = 8;
const LIGHT_BG_COLORS: [&str; BG_COLORS_LENGTH] = [
    "#f9b5b5", "#9cf2dc", "#9becf2", "#6ab6e9", "#cabff9", "#dbb3ef", "#f2cdea", "#f99daf",
];
const DARK_BG_COLORS: [&str; BG_COLORS_LENGTH] = [
    "#602323", "#187f65", "#1a5055", "#1a4563", "#312569", "#5b3171", "#633057", "#652e39",
];

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

async fn send_word_of_the_day(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    channel_id: i64,
) -> ResponseResult<()> {
    if let Ok(image) = get_image("lemma", "etymology") {
        bot.send_photo(ChatId(channel_id), InputFile::memory(image))
            .await?;
    }

    Ok(())
}

/// Schedule the execution of `send_word_of_the_day`
///
/// # Arguments
///
/// * `db_handler` - Handler fot the database
/// * `bot` - The bot
/// * `hour` - Hour of the day to schedule the execution
/// * `min` - Minute of the day to schedule the execution
///
pub async fn schedule_word_of_the_day(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    hour: u32,
    min: u32,
) {
    let channel_id = std::env::var("WOTD_CHANNEL_ID")
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let target_moment = Local::now()
        .with_hour(hour)
        .unwrap()
        .with_minute(min)
        .unwrap()
        .with_second(0)
        .unwrap();

    let mut initial_delay = target_moment - Local::now();

    if initial_delay < Duration::days(0) {
        initial_delay = initial_delay + Duration::days(1);
    }

    let initial_delay = StdDuration::from_secs(initial_delay.num_seconds().try_into().unwrap());

    let mut interval = interval_at(
        Instant::now() + initial_delay,
        StdDuration::from_secs(SECONDS_IN_A_DAY),
    );

    log::info!("Word of the days broadcast scheduled!");

    loop {
        interval.tick().await;
        send_word_of_the_day(db_handler.clone(), bot.clone(), channel_id)
            .await
            .unwrap_or_else(|error| {
                log::warn!("Error while sending word of the day {:?}", error);
            });
    }
}
