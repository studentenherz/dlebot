mod database;
mod handle_inline;
mod handle_message;

use dotenvy::dotenv;
use teloxide::prelude::*;

use database::DatabaseHandler;
use handle_inline::handle_inline;
use handle_message::handle_message;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_handler = DatabaseHandler::from_env().await;

    let x = db_handler.get_list_like("ho").await;
    println!("{:?}", x);

    if let Some(x) = db_handler.get_exact("cisco").await {
        println!("{:?}", x);
    }

    if let Some(x) = db_handler.get_random().await {
        println!("{:?}", x);
    }

    // pretty_env_logger::init();
    // log::info!("Starting command bot...");

    // let bot = Bot::from_env();

    // let handler = dptree::entry()
    //     .branch(Update::filter_message().endpoint(handle_message))
    //     .branch(Update::filter_inline_query().endpoint(handle_inline));

    // Dispatcher::builder(bot, handler)
    //     .enable_ctrlc_handler()
    //     .build()
    //     .dispatch()
    //     .await;
}
