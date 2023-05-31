mod handle_message;

use dotenvy::dotenv;
use handle_message::handle_messages;
use teloxide::{
    prelude::*,
    types::{InlineQueryResult, InlineQueryResultArticle},
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_messages))
        .branch(Update::filter_inline_query().endpoint(handle_inline));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn handle_inline(bot: Bot, q: InlineQuery) -> ResponseResult<()> {
    let mut results: Vec<InlineQueryResult> = vec![];

    for i in 1..10 {
        results.push(InlineQueryResult::Article(InlineQueryResultArticle::new(
            i.to_string(),
            i.to_string(),
            teloxide::types::InputMessageContent::Text(
                teloxide::types::InputMessageContentText::new(i.to_string()),
            ),
        )));
    }

    bot.answer_inline_query(q.id, results).await?;

    Ok(())
}
