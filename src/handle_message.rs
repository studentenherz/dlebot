use teloxide::{
    adaptors::DefaultParseMode,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, Me},
    utils::command::BotCommands,
};

use crate::database::DatabaseHandler;
use crate::utils::{smart_split, MAX_MASSAGE_LENGTH};

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

pub async fn set_commands(bot: DefaultParseMode<Bot>) -> ResponseResult<()> {
    bot.set_my_commands(Command::bot_commands()).await?;

    Ok(())
}

const KEY_RANDOM: &str = "🎲 Palabra aleatoria";
const KEY_WOTD: &str = "📖 Palabra del día";
const KEY_SUBSCRIPTION: &str = "🔔 Suscripción";
const KEY_HELP: &str = "❔ Ayuda";

async fn send_start(bot: DefaultParseMode<Bot>, msg: Message) -> ResponseResult<()> {
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

    bot.send_message(msg.chat.id, include_str!("templates/start.txt"))
        .disable_web_page_preview(true)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn send_help(bot: DefaultParseMode<Bot>, msg: Message, me: Me) -> ResponseResult<()> {
    let inline_keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::switch_inline_query(
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

    Ok(())
}

async fn send_random(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    msg: Message,
) -> ResponseResult<()> {
    if let Some(result) = db_handler.get_random().await {
        bot.send_message(msg.chat.id, result.definition).await?;
    }

    Ok(())
}

pub async fn handle_message(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    msg: Message,
    me: Me,
) -> ResponseResult<()> {
    match msg.via_bot {
        Some(via_bot) if via_bot.id == me.id => return Ok(()),

        _ => {
            if let Some(text) = msg.text() {
                match BotCommands::parse(text, me.username()) {
                    Ok(Command::Start) => {
                        send_start(bot, msg).await?;
                    }

                    Ok(Command::Help | Command::Ayuda) => {
                        send_help(bot, msg, me).await?;
                    }

                    Ok(Command::Aleatorio) => {
                        send_random(db_handler, bot, msg).await?;
                    }

                    Ok(_) => {
                        bot.send_message(msg.chat.id, "Not implemented command")
                            .await?;
                    }

                    Err(_) => match text {
                        KEY_RANDOM => {
                            send_random(db_handler, bot, msg).await?;
                        }
                        KEY_HELP => {
                            send_help(bot, msg, me).await?;
                        }
                        KEY_SUBSCRIPTION => {
                            bot.send_message(msg.chat.id, KEY_SUBSCRIPTION).await?;
                        }
                        KEY_WOTD => {
                            bot.send_message(msg.chat.id, KEY_WOTD).await?;
                        }
                        _ => match db_handler.get_exact(text).await {
                            Some(result) => {
                                for definition in
                                    smart_split(&result.definition, MAX_MASSAGE_LENGTH)
                                {
                                    bot.send_message(msg.chat.id, definition).await?;
                                }
                            }
                            None => {
                                let url = match reqwest::Url::parse(&format!(
                                    "https://dle.rae.es/{}",
                                    text
                                )) {
                                    Ok(value) => value,
                                    Err(_) => reqwest::Url::parse("https://dle.rae.es/").unwrap(),
                                };

                                let inline_keyboard = InlineKeyboardMarkup::new([[
                                    InlineKeyboardButton::switch_inline_query_current_chat(
                                        "Probar inline",
                                        "",
                                    ),
                                    InlineKeyboardButton::url("Buscar en dle.rae.es", url),
                                ]]);

                                bot.send_message(
                                    msg.chat.id,
                                    format!(include_str!("templates/not_found.txt"), text),
                                )
                                .reply_markup(inline_keyboard)
                                .await?;
                            }
                        },
                    },
                };
            }
        }
    }
    Ok(())
}
