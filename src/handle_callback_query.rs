use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, User},
};

use crate::{
    database::DatabaseHandler,
    utils::{DESUBS_CALLBACK_DATA, SUBS_CALLBACK_DATA},
    DLEBot,
};

pub async fn handle_callback_query(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    query: CallbackQuery,
) -> ResponseResult<()> {
    if let Ok(user_id) = query.from.id.0.try_into() {
        async fn edit_message(
            subscribed: bool,
            message: Message,
            user: User,
            bot: DLEBot,
        ) -> ResponseResult<()> {
            let inline_keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
                if subscribed {
                    "Desuscribirme"
                } else {
                    "¡Suscribirme!"
                },
                if subscribed {
                    DESUBS_CALLBACK_DATA
                } else {
                    SUBS_CALLBACK_DATA
                },
            )]]);

            let chat_id = message.chat.id;
            let message_id = message.id;

            bot.clone()
                .edit_message_text(
                    chat_id,
                    message_id,
                    format!(
                        include_str!("templates/subscription.txt"),
                        user.first_name,
                        if subscribed { "SÍ" } else { "NO" }
                    ),
                )
                .await?;

            bot.edit_message_reply_markup(chat_id, message_id)
                .reply_markup(inline_keyboard)
                .await?;

            Ok(())
        }

        match query.data.as_deref() {
            Some(SUBS_CALLBACK_DATA) => {
                bot.answer_callback_query(query.id.clone())
                    .text("✅ ¡Te has suscrito!")
                    .await?;
                if let Some(message) = query.message {
                    edit_message(true, message, query.from, bot).await?;
                }
                db_handler.set_subscribed(user_id, true).await;
                db_handler
                    .add_callback_query_event(user_id, SUBS_CALLBACK_DATA.to_string())
                    .await;
            }
            Some(DESUBS_CALLBACK_DATA) => {
                bot.answer_callback_query(query.id)
                    .text("❌ Te has desuscrito.")
                    .await?;
                if let Some(message) = query.message {
                    edit_message(false, message, query.from, bot).await?;
                }
                db_handler.set_subscribed(user_id, false).await;
                db_handler
                    .add_callback_query_event(user_id, DESUBS_CALLBACK_DATA.to_string())
                    .await;
            }
            _ => {
                bot.answer_callback_query(&query.id).await?;
                log::warn!("Unrecognized callback query: {:?}", query);
            }
        }
    }

    Ok(())
}
