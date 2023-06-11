use teloxide::{adaptors::DefaultParseMode, prelude::*};

use crate::database::DatabaseHandler;

async fn broadcast(
    message: String,
    users: Vec<i64>,
    bot: DefaultParseMode<Bot>,
) -> ResponseResult<()> {
    async fn send_message(
        user_id: i64,
        bot: DefaultParseMode<Bot>,
        wotd: String,
    ) -> ResponseResult<()> {
        bot.send_message(
            ChatId(user_id),
            format!("📖 Palabra del día\n\n {}", wotd.clone().trim()),
        )
        .await?;

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
    bot: DefaultParseMode<Bot>,
) -> ResponseResult<()> {
    broadcast(
        db_handler.get_word_of_the_day().await,
        db_handler.get_subscribed_list().await,
        bot,
    )
    .await
}
