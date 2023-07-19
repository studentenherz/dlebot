use ::teloxide::prelude::*;
use chrono::{offset::Local, Duration, Timelike};
use tokio::time::{interval_at, Duration as StdDuration, Instant};

use crate::{database::DatabaseHandler, image::send_image, DLEBot};

const SECONDS_IN_A_DAY: u64 = 24 * 60 * 60;

async fn send_word_of_the_day(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    chat_id: ChatId,
) -> ResponseResult<()> {
    if let Ok(wotd) = db_handler.get_word_of_the_day().await {
        send_image(wotd, bot, chat_id).await?;
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
        send_word_of_the_day(db_handler.clone(), bot.clone(), ChatId(channel_id))
            .await
            .unwrap_or_else(|error| {
                log::warn!("Error while sending word of the day {:?}", error);
            });
    }
}
