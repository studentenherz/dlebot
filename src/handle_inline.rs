use teloxide::{
    adaptors::DefaultParseMode,
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
    },
};

use crate::database::DatabaseHandler;
use crate::utils::{smart_split, MAX_MASSAGE_LENGTH};

pub async fn handle_inline(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    q: InlineQuery,
) -> ResponseResult<()> {
    let words = db_handler.get_list_like(&q.query).await;

    let mut results: Vec<InlineQueryResult> = vec![];

    for word in words {
        for (id, &part) in smart_split(&word.definition, MAX_MASSAGE_LENGTH)
            .iter()
            .enumerate()
        {
            results.push(InlineQueryResult::Article(
                InlineQueryResultArticle::new(
                    format!("{}_{}", &word.lemma, id),
                    &word.lemma,
                    InputMessageContent::Text(InputMessageContentText::new(part)),
                )
                .description(part),
            ));
        }
    }

    bot.answer_inline_query(q.id, results).await?;

    Ok(())
}
