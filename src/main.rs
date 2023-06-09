mod database;
mod handle_inline;
mod handle_message;
mod utils;

use dotenvy::dotenv;
use teloxide::prelude::*;

use database::DatabaseHandler;
use handle_inline::handle_inline;
use handle_message::{handle_message, set_commands};

#[tokio::main]
async fn main() -> ResponseResult<()> {
    dotenv().ok();

    let db_handler = DatabaseHandler::from_env().await;

    pretty_env_logger::init();

    let url = std::env::var("TELEGRAM_BOT_API_URL").unwrap();
    let url = reqwest::Url::parse(&url).unwrap();

    log::info!("Starting DLE bot with server {}...", url.as_str());

    let bot = Bot::from_env()
        .set_api_url(url)
        .parse_mode(teloxide::types::ParseMode::Html);

    set_commands(bot.clone()).await?;

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_inline_query().endpoint(handle_inline));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db_handler])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
