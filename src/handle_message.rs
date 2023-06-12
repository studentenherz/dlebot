use teloxide::{
    adaptors::DefaultParseMode,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, Me, User},
    utils::command::BotCommands,
};

use crate::database::DatabaseHandler;
use crate::utils::{
    base64_decode, base64_encode, smart_split, DESUBS_CALLBACK_DATA, MAX_MASSAGE_LENGTH,
    SUBS_CALLBACK_DATA,
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Inicia el bot")]
    Start(String),
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

async fn send_word_of_the_day(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    msg: Message,
) -> ResponseResult<()> {
    let definition = db_handler.get_word_of_the_day().await;
    bot.send_message(
        msg.chat.id,
        format!("📖 Palabra del día\n\n {}", definition.trim()),
    )
    .await?;

    Ok(())
}

async fn send_subscription(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    msg: Message,
    user: &User,
) -> ResponseResult<()> {
    let subscribed = match db_handler.get_user(user.id.0.try_into().unwrap()).await {
        Some(user) => user.subscribed,
        None => false,
    };

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

    bot.send_message(
        msg.chat.id,
        format!(
            include_str!("templates/subscription.txt"),
            user.first_name,
            if subscribed { "SÍ" } else { "NO" }
        ),
    )
    .reply_markup(inline_keyboard)
    .await?;

    Ok(())
}

pub async fn send_message(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    msg: Message,
    user: &User,
    text: &str,
    me: Me,
) -> ResponseResult<()> {
    match db_handler.get_exact(text).await {
        Some(result) => {
            for (index, &definition) in smart_split(&result.definition, MAX_MASSAGE_LENGTH)
                .iter()
                .enumerate()
            {
                let definition = if index == 0 {
                    definition.replacen(
                        &result.lemma,
                        &format!(
                            r#"<a href="https://t.me/{}?start={}">{}</a>"#,
                            me.username(),
                            base64_encode(result.lemma.to_string()),
                            result.lemma
                        ),
                        1,
                    )
                } else {
                    definition.to_string()
                };
                bot.send_message(msg.chat.id, definition)
                    .disable_web_page_preview(true)
                    .await?;
            }

            db_handler
                .add_sent_definition_event(
                    user.id.0.try_into().unwrap(),
                    msg.date.into(),
                    result.lemma,
                )
                .await;
        }
        None => {
            let fuzzy_list = db_handler.get_fuzzy_list(text).await;

            let similar_words = if fuzzy_list.is_empty() {
                "".to_string()
            } else {
                let list = fuzzy_list
                    .iter()
                    .map(|x| {
                        format!(
                            r#"<a href="https://t.me/{}?start={}">{}</a>"#,
                            me.username(),
                            base64_encode(x.to_string()),
                            x
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n— ");
                format!("Estas son algunas entradas parecidas:\n\n— {}", list)
            };

            let url = match reqwest::Url::parse(&format!("https://dle.rae.es/{}", text)) {
                Ok(value) => value,
                Err(_) => reqwest::Url::parse("https://dle.rae.es/").unwrap(),
            };

            let inline_keyboard = InlineKeyboardMarkup::new([[
                InlineKeyboardButton::switch_inline_query_current_chat("Probar inline", ""),
                InlineKeyboardButton::url("Buscar en dle.rae.es", url),
            ]]);

            bot.send_message(
                msg.chat.id,
                format!(include_str!("templates/not_found.txt"), text, similar_words),
            )
            .disable_web_page_preview(true)
            .reply_markup(inline_keyboard)
            .await?;
        }
    }

    Ok(())
}

pub async fn handle_message(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    msg: Message,
    me: Me,
) -> ResponseResult<()> {
    let user = msg.from().unwrap().clone();

    db_handler
        .set_in_bot(user.id.0.try_into().unwrap(), true)
        .await;

    match msg.via_bot {
        Some(via_bot) if via_bot.id == me.id => return Ok(()),

        _ => {
            db_handler
                .add_message_event(
                    user.id.0.try_into().unwrap(),
                    msg.date.into(),
                    msg.text().unwrap_or("").to_string(),
                )
                .await;

            if let Some(text) = msg.clone().text() {
                match BotCommands::parse(text, me.username()) {
                    Ok(Command::Start(start_parameter)) => {
                        match base64_decode(start_parameter.clone()) {
                            Ok(decoded) => match decoded.as_ref() {
                                "" => {
                                    send_start(bot, msg).await?;
                                }
                                _ => {
                                    send_message(db_handler, bot, msg, &user, &decoded, me).await?;
                                }
                            },
                            _ => {
                                log::warn!("Failed to decode start_parameter {}", start_parameter);
                                send_start(bot, msg).await?;
                            }
                        }
                    }

                    Ok(Command::Help | Command::Ayuda) => {
                        send_help(bot, msg, me).await?;
                    }

                    Ok(Command::Aleatorio) => {
                        send_random(db_handler, bot, msg).await?;
                    }

                    Ok(Command::Pdd) => {
                        send_word_of_the_day(db_handler, bot, msg).await?;
                    }

                    Ok(Command::Suscripcion) => {
                        send_subscription(db_handler, bot, msg, &user).await?;
                    }

                    Err(_) => match text {
                        KEY_RANDOM => {
                            send_random(db_handler, bot, msg).await?;
                        }
                        KEY_HELP => {
                            send_help(bot, msg, me).await?;
                        }
                        KEY_SUBSCRIPTION => {
                            send_subscription(db_handler, bot, msg, &user).await?;
                        }
                        KEY_WOTD => {
                            send_word_of_the_day(db_handler, bot, msg).await?;
                        }
                        _ => {
                            send_message(db_handler, bot, msg, &user, text, me).await?;
                        }
                    },
                };
            }
        }
    }
    Ok(())
}

pub async fn handle_edited_message(
    db_handler: DatabaseHandler,
    bot: DefaultParseMode<Bot>,
    msg: Message,
) -> ResponseResult<()> {
    let user = msg.from().unwrap().clone();

    db_handler
        .set_in_bot(user.id.0.try_into().unwrap(), true)
        .await;

    if let Some(text) = msg.text() {
        db_handler
            .add_edited_message_event(
                user.id.0.try_into().unwrap(),
                msg.date.into(),
                msg.text().unwrap_or("").to_string(),
            )
            .await;

        match db_handler.get_exact(text).await {
            Some(result) => {
                for definition in smart_split(&result.definition, MAX_MASSAGE_LENGTH) {
                    bot.send_message(
                        msg.chat.id,
                        format!("😌 ¡Ahora sí!\n\n{}", definition.trim()),
                    )
                    .reply_to_message_id(msg.id)
                    .await?;
                }

                db_handler
                    .add_sent_definition_event(
                        user.id.0.try_into().unwrap(),
                        msg.date.into(),
                        result.lemma,
                    )
                    .await;
            }
            None => {
                let url = match reqwest::Url::parse(&format!("https://dle.rae.es/{}", text)) {
                    Ok(value) => value,
                    Err(_) => reqwest::Url::parse("https://dle.rae.es/").unwrap(),
                };

                let inline_keyboard = InlineKeyboardMarkup::new([[
                    InlineKeyboardButton::switch_inline_query_current_chat("Probar inline", ""),
                    InlineKeyboardButton::url("Buscar en dle.rae.es", url),
                ]]);

                let text = format!(include_str!("templates/not_found.txt"), text, "");

                bot.send_message(msg.chat.id, format!("😐 Así tampoco\n\n{}", text))
                    .reply_markup(inline_keyboard)
                    .reply_to_message_id(msg.id)
                    .await?;
            }
        }
    }

    Ok(())
}
