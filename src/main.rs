mod broadcast;
mod database;
mod handle_callback_query;
mod handle_inline;
mod handle_message;
mod utils;

use dotenvy::dotenv;
use teloxide::{prelude::*, update_listeners::webhooks};

use broadcast::broadcast_word_of_the_day;
use database::DatabaseHandler;
use handle_callback_query::handle_callback_query;
use handle_inline::handle_inline;
use handle_message::{handle_message, set_commands};

#[tokio::main]
async fn main() -> ResponseResult<()> {
    dotenv().ok();

    let db_handler = DatabaseHandler::from_env().await;

    pretty_env_logger::init();

    let api_url = std::env::var("TELEGRAM_BOT_API_URL").unwrap();
    let api_url = reqwest::Url::parse(&api_url).unwrap();

    let bot = Bot::from_env()
        .set_api_url(api_url.clone())
        .parse_mode(teloxide::types::ParseMode::Html);

    let port: u16 = std::env::var("WEBHOOK_PORT").unwrap().parse().unwrap();
    let addr = ([127, 0, 0, 1], port).into();
    let url = std::env::var("WEBHOOK_URL").unwrap();
    let url = reqwest::Url::parse(&url).unwrap();
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url.clone()))
        .await
        .unwrap();

    log::info!(
        "Starting DLE bot: server={}; webhook_url={}",
        api_url.as_str(),
        url.as_str()
    );

    set_commands(bot.clone()).await?;

    broadcast_word_of_the_day(&db_handler, bot.clone()).await?;

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_inline_query().endpoint(handle_inline))
        .branch(Update::filter_callback_query().endpoint(handle_callback_query));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db_handler])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;

    Ok(())
}
