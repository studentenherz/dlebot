use teloxide::{
    payloads::AnswerInlineQuery,
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
        Me, ParseMode,
    },
};

use crate::{
    database::DatabaseHandler,
    utils::{base64_encode, smart_split, MAX_MASSAGE_LENGTH},
    DLEBot,
};

pub async fn handle_inline(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    q: InlineQuery,
    me: Me,
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
            let part = if id == 0 {
                part.to_string()
            } else {
                format!("{}\n{}", &word.lemma, part)
            };

            let part_with_deep_link = part.replacen(
                &word.lemma,
                &format!(
                    r#"<a href="https://t.me/{}?start={}">{}</a>"#,
                    me.username(),
                    base64_encode(word.lemma.clone()),
                    word.lemma
                ),
                1,
            );

            results.push(InlineQueryResult::Article(
                InlineQueryResultArticle::new(
                    format!("{}_{}", &word.lemma, id),
                    &word.lemma,
                    InputMessageContent::Text(
                        InputMessageContentText::new(part_with_deep_link)
                            .disable_web_page_preview(true)
                            .parse_mode(ParseMode::Html),
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
                switch_pm_parameter: Some(base64_encode(q.query)),
                switch_pm_text: Some("No se han encontrado resultados".to_string()),
            },
        )
        .await?;
    } else {
        bot.answer_inline_query(q.id, results).await?;
    }

    Ok(())
}

pub async fn handle_chosen_inline_result(
    db_handler: DatabaseHandler,
    chosen: ChosenInlineResult,
) -> ResponseResult<()> {
    if let Ok(chosen_id) = chosen.from.id.0.try_into() {
        db_handler
            .add_chosen_inline_result_event(chosen_id, chosen.result_id, chosen.query)
            .await;
    }

    Ok(())
}
