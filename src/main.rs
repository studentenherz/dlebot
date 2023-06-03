mod database;
mod handle_inline;
mod handle_message;
mod utils;

use dotenvy::dotenv;
use teloxide::prelude::*;

use database::DatabaseHandler;
use handle_inline::handle_inline;
use handle_message::handle_message;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_handler = DatabaseHandler::from_env().await;

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env().parse_mode(teloxide::types::ParseMode::Html);

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_inline_query().endpoint(handle_inline));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db_handler])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
