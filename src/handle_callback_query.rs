use teloxide::{
    payloads::EditMessageTextSetters,
    prelude::*,
    types::{MaybeInaccessibleMessage, Me, ParseMode},
};

use crate::{
    database::DatabaseHandler,
    settings::{
        settings_keyboard, settings_text, MessageFormat, UserSettings,
        FORMAT_CLASSIC_CALLBACK_DATA, FORMAT_RICH_CALLBACK_DATA, FORMAT_RICH_DEMO_CALLBACK_DATA,
    },
    utils::DISABLED_LINK_PREVIEW,
    DLEBot,
};

/// Store the chosen format. When the button came from the settings menu, the
/// menu is rewritten to show the new one; the rich sample is left as it is.
async fn set_format(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    query: CallbackQuery,
    format: MessageFormat,
    bot_username: &str,
    from_demo: bool,
) -> ResponseResult<()> {
    let user_id = match query.from.id.0.try_into() {
        Ok(user_id) => user_id,
        Err(_) => return Ok(()),
    };

    let settings = UserSettings { format };
    let already_set = db_handler.get_settings(user_id).await == settings;

    bot.answer_callback_query(query.id.clone())
        .text(match (already_set, format) {
            (true, MessageFormat::Rich) => "Ya usas el formato enriquecido",
            (true, MessageFormat::Classic) => "Ya usas el formato clásico",
            (false, MessageFormat::Rich) => "Formato enriquecido activado",
            (false, MessageFormat::Classic) => "Formato clásico activado",
        })
        .await?;

    if already_set {
        return Ok(());
    }

    db_handler.set_message_format(user_id, format).await;
    db_handler
        .add_callback_query_event(user_id, query.data.clone().unwrap_or_default())
        .await;

    if from_demo {
        return Ok(());
    }

    if let Some(MaybeInaccessibleMessage::Regular(message)) = query.message {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            settings_text(bot_username, &query.from.first_name, settings),
        )
        .parse_mode(ParseMode::Html)
        .link_preview_options(DISABLED_LINK_PREVIEW)
        .reply_markup(settings_keyboard())
        .await?;
    }

    Ok(())
}

pub async fn handle_callback_query(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    query: CallbackQuery,
    me: Me,
) -> ResponseResult<()> {
    let data = query.data.clone().unwrap_or_default();

    match data.as_str() {
        FORMAT_RICH_CALLBACK_DATA => {
            set_format(
                db_handler,
                bot,
                query,
                MessageFormat::Rich,
                me.username(),
                false,
            )
            .await?;
        }
        FORMAT_CLASSIC_CALLBACK_DATA => {
            set_format(
                db_handler,
                bot,
                query,
                MessageFormat::Classic,
                me.username(),
                false,
            )
            .await?;
        }
        FORMAT_RICH_DEMO_CALLBACK_DATA => {
            set_format(
                db_handler,
                bot,
                query,
                MessageFormat::Rich,
                me.username(),
                true,
            )
            .await?;
        }
        _ => {
            log::warn!("Unrecognized callback query: {:?}", query);
            bot.answer_callback_query(query.id).await?;
        }
    }

    Ok(())
}
