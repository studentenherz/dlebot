use teloxide::{
    prelude::*,
    types::{InlineQueryResultArticle, Me},
};

use crate::{
    database::DatabaseHandler,
    rich_messages::{
        InlineQueryButton, InlineQueryResultArticleExt, InputMessageContent, InputRichMessage,
        InputRichMessageContent, InputTextMessageContent, RichInlineQueryExt,
        RichInlineQueryResultArticle,
    },
    settings::UserSettings,
    utils::{base64_encode, deep_link, smart_split, MAX_MASSAGE_LENGTH},
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

    let settings: UserSettings = match q.from.id.0.try_into() {
        Ok(user_id) => db_handler.get_settings(user_id).await,
        Err(_) => UserSettings::default(),
    };

    let words = db_handler.get_list_like(&q.query).await;

    let mut results: Vec<RichInlineQueryResultArticle> = vec![];

    for word in words {
        let headword = word.headword().to_string();
        let deep_link_url = deep_link(me.username(), &word.query);
        let html = if settings.is_rich() {
            word.to_html_with_deeplink(&deep_link_url)
        } else {
            word.to_classic_html(Some(&deep_link_url))
        };
        let text = word.to_text();

        let html_parts = smart_split(&html, MAX_MASSAGE_LENGTH);
        let text_parts = smart_split(&text, MAX_MASSAGE_LENGTH);

        for (id, &html_part) in html_parts.iter().enumerate() {
            let html_content = match (id, settings.is_rich()) {
                (0, _) => html_part.to_string(),
                (_, true) => format!(
                    "<h1><a href=\"{}\">{}</a></h1>\n{}",
                    deep_link_url, headword, html_part
                ),
                (_, false) => format!(
                    "<b><a href=\"{}\">{}</a></b>\n\n{}",
                    deep_link_url, headword, html_part
                ),
            };

            let description = text_parts.get(id).copied().unwrap_or_default();
            let content: InputMessageContent = if settings.is_rich() {
                InputRichMessageContent::new(InputRichMessage {
                    html: Some(html_content),
                    markdown: None,
                    is_rtl: None,
                    skip_entity_detection: None,
                })
                .into()
            } else {
                InputTextMessageContent::html(html_content).into()
            };

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
    // Results depend on the user's settings, so they must not be cached for others.
    req.is_personal = Some(true);
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
