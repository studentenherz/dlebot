use teloxide::prelude::*;

use crate::{database::DatabaseHandler, DLEBot};

async fn broadcast(message: String, users: Vec<i64>, bot: DLEBot) -> ResponseResult<()> {
    async fn send_message(user_id: i64, bot: DLEBot, message: String) -> ResponseResult<()> {
        bot.send_message(ChatId(user_id), message).await?;
        Ok(())
    }

    let mut join_set = tokio::task::JoinSet::new();

    for user_id in users {
        join_set.spawn(send_message(user_id, bot.clone(), message.clone()));
    }

    while join_set.join_next().await.is_some() {}

    Ok(())
}

pub async fn broadcast_word_of_the_day(
    db_handler: DatabaseHandler,
    bot: DLEBot,
) -> ResponseResult<()> {
    if let Ok(wotd) = db_handler.get_word_of_the_day().await {
        broadcast(
            format!("📖 Palabra del día\n\n {}", wotd.clone().trim()),
            db_handler.get_subscribed_and_in_bot_list().await,
            bot,
        )
        .await?;
    }

    Ok(())
}
