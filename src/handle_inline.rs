use teloxide::{
    adaptors::DefaultParseMode,
    payloads::AnswerInlineQuery,
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
        ParseMode,
    },
};

use crate::database::DatabaseHandler;
use crate::utils::{smart_split, MAX_MASSAGE_LENGTH};

pub async fn handle_inline(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    q: InlineQuery,
) -> ResponseResult<()> {
    if q.query.is_empty() {
        return Ok(());
    }

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
                    InputMessageContent::Text(
                        InputMessageContentText::new(part).parse_mode(ParseMode::Html),
                    ),
                )
                .description(part),
            ));
        }
    }

    if results.is_empty() {
        <Bot as Requester>::AnswerInlineQuery::new(
            bot.inner().clone(),
            AnswerInlineQuery {
                inline_query_id: q.id,
                results,
                cache_time: None,
                is_personal: None,
                next_offset: None,
                switch_pm_parameter: Some("404".to_string()),
                switch_pm_text: Some("No se han encontrado resultados".to_string()),
            },
        )
        .await?;
    } else {
        bot.answer_inline_query(q.id, results).await?;
    }

    Ok(())
}
