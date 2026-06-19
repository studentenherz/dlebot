use teloxide::{
    payloads::AnswerInlineQuery,
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InlineQueryResultsButton,
        InlineQueryResultsButtonKind, InputMessageContent, InputMessageContentText, Me, ParseMode,
    },
};

use crate::{
    database::DatabaseHandler,
    utils::{base64_encode, smart_split, DISABLED_LINK_PREVIEW, MAX_MASSAGE_LENGTH},
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
        let headword = word.headword().to_string();
        let html = word.to_html();
        let text = word.to_text();
        let deep_link_url = format!(
            "https://t.me/{}?start={}",
            me.username(),
            base64_encode(word.query.clone())
        );

        let html_parts = smart_split(&html, MAX_MASSAGE_LENGTH);
        let text_parts = smart_split(&text, MAX_MASSAGE_LENGTH);

        for (id, &html_part) in html_parts.iter().enumerate() {
            let html_with_link = if id == 0 {
                html_part.replacen(
                    &format!("<b>{}</b>", headword),
                    &format!(r#"<b><a href="{}">{}</a></b>"#, deep_link_url, headword),
                    1,
                )
            } else {
                format!("<b>{}</b>\n{}", headword, html_part)
            };

            let description = text_parts.get(id).copied().unwrap_or_default();

            results.push(InlineQueryResult::Article(
                InlineQueryResultArticle::new(
                    format!("{}_{}", word.query, id),
                    headword.clone(),
                    InputMessageContent::Text(
                        InputMessageContentText::new(html_with_link)
                            .link_preview_options(DISABLED_LINK_PREVIEW)
                            .parse_mode(ParseMode::Html),
                    ),
                )
                .description(description),
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
                button: Some(InlineQueryResultsButton {
                    text: "No se han encontrado resultados".to_string(),
                    kind: InlineQueryResultsButtonKind::StartParameter(base64_encode(q.query)),
                }),
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
