use chrono::{Duration, Local, Timelike};
use tokio::time::{interval_at, Duration as StdDuration, Instant};

use crate::{broadcast::broadcast_word_of_the_day, database::DatabaseHandler, DLEBot};

const SECONDS_IN_A_DAY: u64 = 24 * 60 * 60;

/// Schedule the execution of `broadcast_word_of_the_day`
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
        broadcast_word_of_the_day(db_handler.clone(), bot.clone())
            .await
            .unwrap_or_else(|_| {
                log::warn!("Error while broadcasting word of the day");
            });
    }
}
