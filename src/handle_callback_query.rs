use teloxide::{
    adaptors::DefaultParseMode,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, User},
};

use crate::{
    database::DatabaseHandler,
    utils::{DESUBS_CALLBACK_DATA, SUBS_CALLBACK_DATA},
};

pub async fn handle_callback_query(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    query: CallbackQuery,
) -> ResponseResult<()> {
    async fn edit_message(
        subscribed: bool,
        message: Message,
        user: User,
        bot: DefaultParseMode<Bot>,
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
            db_handler
                .set_subscribed(query.from.id.0.try_into().unwrap(), true)
                .await;
            bot.answer_callback_query(query.id.clone())
                .text("✅ ¡Te has suscrito!")
                .await?;
            edit_message(true, query.message.unwrap(), query.from, bot).await?;
        }
        Some(DESUBS_CALLBACK_DATA) => {
            db_handler
                .set_subscribed(query.from.id.0.try_into().unwrap(), false)
                .await;
            bot.answer_callback_query(query.id)
                .text("❌ Te has desuscrito.")
                .await?;
            edit_message(false, query.message.unwrap(), query.from, bot).await?;
        }
        _ => {
            log::warn!("Unrecognized callback query: {:?}", query);
        }
    }

    Ok(())
}
