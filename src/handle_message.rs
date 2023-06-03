use teloxide::{
    adaptors::DefaultParseMode,
    prelude::*,
    types::{
        ChatAction, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, Me,
    },
    utils::command::BotCommands,
};

use crate::database::DatabaseHandler;
use crate::utils::smart_split;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Inicia el bot")]
    Start,
    #[command(description = "Ayuda")]
    Help,
    #[command(description = "Cómo usarlo")]
    Ayuda,
    #[command(description = "Buscar término aleatorio")]
    Aleatorio,
    #[command(description = "Mostrar la «Palabra del día»")]
    Pdd,
    #[command(description = "Suscribir a la «Palabra del día»")]
    Suscripcion,
}

const KEY_RANDOM: &str = "🎲 Palabra aleatoria";
const KEY_WOTD: &str = "📖 Palabra del día";
const KEY_SUBSCRIPTION: &str = "🔔 Suscripción";
const KEY_HELP: &str = "❔ Ayuda";
const MAX_MASSAGE_LENGTH: usize = 4096;

pub async fn handle_message(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    msg: Message,
    me: Me,
) -> ResponseResult<()> {
    let keyboard = KeyboardMarkup::new([
        [
            KeyboardButton::new(KEY_RANDOM),
            KeyboardButton::new(KEY_WOTD),
        ],
        [
            KeyboardButton::new(KEY_SUBSCRIPTION),
            KeyboardButton::new(KEY_HELP),
        ],
    ])
    .resize_keyboard(true);

    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Start) => {
                bot.send_message(msg.chat.id, include_str!("templates/start.txt"))
                    .disable_web_page_preview(true)
                    .reply_markup(keyboard)
                    .await?;
            }

            Ok(Command::Help | Command::Ayuda) => {
                let inline_keyboard =
                    InlineKeyboardMarkup::new([[InlineKeyboardButton::switch_inline_query(
                        "Buscar definición",
                        "",
                    )]]);

                bot.send_message(
                    msg.chat.id,
                    format!(
                        include_str!("templates/help.txt"),
                        bot_username = me.username()
                    ),
                )
                .reply_markup(inline_keyboard)
                .await?;
            }

            Ok(_) => {
                bot.send_message(msg.chat.id, "Not implemented command")
                    .await?;
            }

            Err(_) => match text {
                KEY_RANDOM => {
                    bot.send_message(msg.chat.id, KEY_RANDOM).await?;
                }
                KEY_HELP => {
                    bot.send_message(msg.chat.id, KEY_HELP).await?;
                }
                KEY_SUBSCRIPTION => {
                    bot.send_message(msg.chat.id, KEY_SUBSCRIPTION).await?;
                }
                KEY_WOTD => {
                    bot.send_message(msg.chat.id, KEY_WOTD).await?;
                }
                _ => {
                    bot.send_chat_action(msg.chat.id, ChatAction::Typing)
                        .await?;

                    match db_handler.get_exact(text).await {
                        Some(result) => {
                            for definition in smart_split(&result.definition, MAX_MASSAGE_LENGTH) {
                                bot.send_message(msg.chat.id, definition).await?;
                            }
                        }
                        None => {
                            bot.send_message(
                                msg.chat.id,
                                format!(include_str!("templates/not_found.txt"), text),
                            )
                            .await?;
                        }
                    }
                }
            },
        };
    }
    Ok(())
}
