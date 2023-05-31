use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
    },
};

pub async fn handle_inline(bot: Bot, q: InlineQuery) -> ResponseResult<()> {
    let mut results: Vec<InlineQueryResult> = vec![];

    for i in 1..10 {
        results.push(InlineQueryResult::Article(InlineQueryResultArticle::new(
            i.to_string(),
            i.to_string(),
            InputMessageContent::Text(InputMessageContentText::new(i.to_string())),
        )));
    }

    bot.answer_inline_query(q.id, results).await?;

    Ok(())
}
