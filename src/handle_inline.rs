use teloxide::{
    prelude::*,
    types::{InlineQueryResultArticle, Me},
};

use crate::{
    database::DatabaseHandler,
    rich_messages::{
        InlineQueryButton, InlineQueryResultArticleExt, InputRichMessage, InputRichMessageContent,
        RichInlineQueryExt, RichInlineQueryResultArticle,
    },
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

    let mut results: Vec<RichInlineQueryResultArticle> = vec![];

    for word in words {
        let headword = word.headword().to_string();
        let deep_link_url = format!(
            "https://t.me/{}?start={}",
            me.username(),
            base64_encode(&word.query)
        );
        let html = word.to_html(Some(&deep_link_url));
        let text = word.to_text();

        let html_parts = smart_split(&html, MAX_MASSAGE_LENGTH);
        let text_parts = smart_split(&text, MAX_MASSAGE_LENGTH);

        for (id, &html_part) in html_parts.iter().enumerate() {
            let html_content = if id == 0 {
                html_part.to_string()
            } else {
                format!(
                    "<h1><a href=\"{}\">{}</a></h1>\n{}",
                    deep_link_url, headword, html_part
                )
            };

            let description = text_parts.get(id).copied().unwrap_or_default();
            let content = InputRichMessageContent::new(InputRichMessage {
                html: Some(html_content),
                markdown: None,
                is_rtl: None,
                skip_entity_detection: None,
            });

            results.push(
                InlineQueryResultArticle::new_with_rich_message(
                    format!("{}_{}", word.query, id),
                    headword.clone(),
                    content,
                )
                .description(description),
            );
        }
    }

    let mut req = bot.answer_inline_query_rich(q.id, results);
    if req.results.is_empty() {
        req.button = Some(InlineQueryButton::start_parameter(
            "No se han encontrado resultados",
            base64_encode(&q.query),
        ));
    }
    req.await?;

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
