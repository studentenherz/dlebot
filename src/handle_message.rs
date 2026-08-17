use chrono::NaiveDate;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, Me, MessageId,
        ParseMode, ReplyParameters,
    },
    utils::command::{BotCommands, ParseError},
};

use crate::{
    broadcast::broadcast_for_all,
    database::DatabaseHandler,
    dle_ir::DleWord,
    image::send_image,
    rich_messages::{InputRichMessage, RichMessageExt},
    settings::{
        send_format_demo, send_settings, MessageFormat, UserSettings, START_DEMO, START_SETTINGS,
    },
    utils::{
        base64_decode, deep_link, smart_split, DISABLED_LINK_PREVIEW,
        MAX_MASSAGE_LENGTH,
    },
    DLEBot,
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
    #[command(description = "Configurar el bot")]
    Ajustes,
    // #[command(description = "Mostrar la «Palabra del día»")]
    // Pdd,
}

fn split_by_first_whitespace(text: String) -> Result<(String, String), ParseError> {
    let split: Vec<&str> = text.split(' ').collect();
    if split.len() >= 2 {
        return Ok((split[0].to_string(), split[1..].join(" ")));
    }

    Err(ParseError::TooFewArguments {
        expected: 2,
        found: 1,
        message: "/command arg1 arg2".to_string(),
    })
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum AdminCommand {
    #[command(description = "Envía un mensaje a todos")]
    Broadcast(String),
    #[command(description = "Envía definición con una imagen")]
    Image(String),
    #[command(
        description = "Setea la palabra del día de una fecha",
        parse_with = split_by_first_whitespace
    )]
    SetPdd { date: String, lemma: String },
    #[command(description = "Obtén la lista de palabras programadas")]
    GetSchedule,
}

pub async fn set_commands(bot: DLEBot) -> ResponseResult<()> {
    bot.set_my_commands(Command::bot_commands()).await?;
    Ok(())
}

const KEY_RANDOM: &str = "🎲 Palabra aleatoria";
// const KEY_WOTD: &str = "📖 Palabra del día";
const KEY_HELP: &str = "❔ Ayuda";
const KEY_SETTINGS: &str = "⚙️ Ajustes";

async fn send_start(bot: DLEBot, msg: Message) -> ResponseResult<()> {
    let keyboard = KeyboardMarkup::new([[
        KeyboardButton::new(KEY_RANDOM),
        KeyboardButton::new(KEY_HELP),
        KeyboardButton::new(KEY_SETTINGS),
        // KeyboardButton::new(KEY_WOTD),
    ]])
    // .append_row([KeyboardButton::new(KEY_HELP)])
    .resize_keyboard();

    bot.send_message(msg.chat.id, include_str!("templates/start.txt"))
        .parse_mode(ParseMode::Html)
        .link_preview_options(DISABLED_LINK_PREVIEW)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn send_help(bot: DLEBot, msg: Message, me: Me) -> ResponseResult<()> {
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
    .parse_mode(ParseMode::Html)
    .reply_markup(inline_keyboard)
    .await?;

    Ok(())
}

async fn send_random(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    msg: Message,
    me: Me,
    settings: UserSettings,
) -> ResponseResult<()> {
    if let Some(result) = db_handler.get_random().await {
        send_result(bot, &msg, me, &result, settings).await?;
    }
    Ok(())
}

// async fn send_word_of_the_day(
//     db_handler: DatabaseHandler,
//     bot: DLEBot,
//     msg: Message,
// ) -> ResponseResult<()> {
//     if let Ok(wotd) = db_handler.get_word_of_the_day().await {
//         bot.send_message(
//             msg.chat.id,
//             format!("📖 Palabra del día\n\n {}", wotd.to_html(None).trim()),
//         )
//         .link_preview_options(DISABLED_LINK_PREVIEW)
//         .await?;
//     }
//     Ok(())
// }

/// Send `word` in the format the user chose in their settings, optionally
/// prefixed with a line of text and replying to a message.
async fn send_definition(
    bot: DLEBot,
    chat_id: ChatId,
    word: &DleWord,
    deep_link_url: &str,
    settings: UserSettings,
    prefix: Option<&str>,
    reply_to: Option<MessageId>,
) -> ResponseResult<()> {
    match settings.format {
        MessageFormat::Rich => {
            let html = match prefix {
                Some(prefix) => format!(
                    "<p>{}</p>{}",
                    prefix,
                    word.to_html_with_deeplink(deep_link_url)
                ),
                None => word.to_html_with_deeplink(deep_link_url),
            };

            let mut request = bot.send_rich_message(
                chat_id,
                InputRichMessage {
                    markdown: None,
                    html: Some(html),
                    is_rtl: None,
                    skip_entity_detection: None,
                },
            );
            if let Some(message_id) = reply_to {
                request.reply_parameters = Some(ReplyParameters::new(message_id));
            }
            request.await?;
        }

        MessageFormat::Classic => {
            let html = word.to_classic_html(Some(deep_link_url));
            let html = match prefix {
                Some(prefix) => format!("{}\n\n{}", prefix, html),
                None => html,
            };

            for part in smart_split(&html, MAX_MASSAGE_LENGTH) {
                let mut request = bot
                    .send_message(chat_id, part.trim())
                    .parse_mode(ParseMode::Html)
                    .link_preview_options(DISABLED_LINK_PREVIEW);
                if let Some(message_id) = reply_to {
                    request = request.reply_parameters(ReplyParameters::new(message_id));
                }
                request.await?;
            }
        }
    }

    Ok(())
}

async fn send_result(
    bot: DLEBot,
    msg: &Message,
    me: Me,
    word: &DleWord,
    settings: UserSettings,
) -> ResponseResult<()> {
    let deep_link_url = deep_link(me.username(), &word.query);

    send_definition(bot, msg.chat.id, word, &deep_link_url, settings, None, None).await
}

pub async fn send_message(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    msg: Message,
    user_id: i64,
    text: &str,
    me: Me,
    settings: UserSettings,
) -> ResponseResult<()> {
    match db_handler.get_exact(text).await {
        Some(result) => {
            send_result(bot, &msg, me, &result, settings).await?;

            db_handler
                .add_sent_definition_event(user_id, msg.date.into(), result.query)
                .await;
        }
        None => {
            let fuzzy_list = db_handler.get_fuzzy_list(text).await;

            let similar_words = if fuzzy_list.is_empty() {
                "".to_string()
            } else {
                let list = fuzzy_list
                    .iter()
                    .map(|x| format!(r#"<a href="{}">{}</a>"#, deep_link(me.username(), x), x))
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
            .parse_mode(ParseMode::Html)
            .link_preview_options(DISABLED_LINK_PREVIEW)
            .reply_markup(inline_keyboard)
            .await?;
        }
    }

    Ok(())
}

pub async fn handle_message(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    msg: Message,
    me: Me,
) -> ResponseResult<()> {
    if let Some(user) = msg.clone().from {
        if let Ok(user_id) = user.id.0.try_into() {
            db_handler.set_in_bot(user_id, true).await;

            match msg.via_bot {
                Some(via_bot) if via_bot.id == me.id => return Ok(()),

                _ => {
                    db_handler
                        .add_message_event(
                            user_id,
                            msg.date.into(),
                            msg.text().unwrap_or("").to_string(),
                        )
                        .await;

                    if let Some(text) = msg.clone().text() {
                        match BotCommands::parse(text, me.username()) {
                            Ok(AdminCommand::Broadcast(message))
                                if db_handler.is_admin(user_id).await =>
                            {
                                broadcast_for_all(message, db_handler, bot).await?;
                                return Ok(());
                            }
                            Ok(AdminCommand::Image(lemma))
                                if db_handler.is_admin(user_id).await =>
                            {
                                if let Some(word) = db_handler.get_exact(&lemma).await {
                                    send_image(word, bot, ChatId(user_id), false).await?;
                                } else {
                                    bot.send_message(
                                        ChatId(user_id),
                                        format!("No encontré {}", lemma),
                                    )
                                    .await?;
                                }
                                return Ok(());
                            }
                            Ok(AdminCommand::SetPdd { date, lemma })
                                if db_handler.is_admin(user_id).await =>
                            {
                                if let Ok(date) = NaiveDate::parse_from_str(&date, "%d/%m/%Y") {
                                    match db_handler.set_word_of_the_day(&lemma, date).await {
                                        Ok(true) => {
                                            bot.send_message(
                                                msg.chat.id,
                                                format!("✅ {}: {}", lemma, date),
                                            )
                                            .await?;
                                        }
                                        Ok(false) => {
                                            bot.send_message(
                                                msg.chat.id,
                                                format!("No se encontró la palabra {}", lemma),
                                            )
                                            .await?;
                                        }
                                        Err(err) => {
                                            bot.send_message(
                                                msg.chat.id,
                                                format!("Hubo un error accediendo a la base de datos: <pre>{}</pre>", err),
                                            )
                                            .await?;
                                        }
                                    }
                                } else {
                                    bot.send_message(msg.chat.id, "El formato de la fecha es <pre>%d/%m/%Y</pre> (por ejemplo: 17/7/1997)").await?;
                                }
                                return Ok(());
                            }
                            Ok(AdminCommand::GetSchedule) if db_handler.is_admin(user_id).await => {
                                match db_handler.get_word_of_the_day_schedule().await {
                                    Ok(schedule) => {
                                        let mut text = String::new();
                                        for wotd in schedule {
                                            text += &format!(
                                                "<pre>{}: {}</pre>\n",
                                                wotd.date.unwrap_or_default(),
                                                wotd.lemma
                                            );
                                        }
                                        bot.send_message(msg.chat.id, text).await?;
                                    }
                                    Err(error) => {
                                        bot.send_message(
                                            msg.chat.id,
                                            format!(
                                                "Hubo un error con la base de datos: {}",
                                                error
                                            ),
                                        )
                                        .await?;
                                    }
                                }
                                return Ok(());
                            }
                            _ => {}
                        }

                        let settings = db_handler.get_settings(user_id).await;

                        match BotCommands::parse(text, me.username()) {
                            Ok(Command::Start(start_parameter)) => {
                                match base64_decode(start_parameter.clone()) {
                                    Ok(decoded) => match decoded.as_ref() {
                                        "" => {
                                            send_start(bot, msg).await?;
                                        }
                                        START_SETTINGS => {
                                            send_settings(
                                                me.username(),
                                                &bot,
                                                msg.chat.id,
                                                &user.first_name,
                                                settings,
                                            )
                                            .await?;
                                        }
                                        START_DEMO => {
                                            send_format_demo(&bot, msg.chat.id).await?;
                                        }
                                        _ => {
                                            send_message(
                                                db_handler, bot, msg, user_id, &decoded, me,
                                                settings,
                                            )
                                            .await?;
                                        }
                                    },
                                    _ => {
                                        log::warn!(
                                            "Failed to decode start_parameter {}",
                                            start_parameter
                                        );
                                        send_start(bot, msg).await?;
                                    }
                                }
                            }

                            Ok(Command::Help | Command::Ayuda) => {
                                send_help(bot, msg, me).await?;
                            }

                            Ok(Command::Aleatorio) => {
                                send_random(db_handler, bot, msg, me, settings).await?;
                            }

                            Ok(Command::Ajustes) => {
                                send_settings(
                                    me.username(),
                                    &bot,
                                    msg.chat.id,
                                    &user.first_name,
                                    settings,
                                )
                                .await?;
                            }

                            // Ok(Command::Pdd) => {
                            //     send_word_of_the_day(db_handler, bot, msg).await?;
                            // }
                            Err(_) => match text {
                                KEY_RANDOM => {
                                    send_random(db_handler, bot, msg, me, settings).await?;
                                }
                                KEY_HELP => {
                                    send_help(bot, msg, me).await?;
                                }
                                KEY_SETTINGS => {
                                    send_settings(
                                        me.username(),
                                        &bot,
                                        msg.chat.id,
                                        &user.first_name,
                                        settings,
                                    )
                                    .await?;
                                }
                                // KEY_WOTD => {
                                //     send_word_of_the_day(db_handler, bot, msg).await?;
                                // }
                                _ => {
                                    send_message(db_handler, bot, msg, user_id, text, me, settings)
                                        .await?;
                                }
                            },
                        };
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_edited_message(
    db_handler: DatabaseHandler,
    bot: DLEBot,
    msg: Message,
    me: Me,
) -> ResponseResult<()> {
    if let Some(user) = msg.clone().from {
        if let Ok(user_id) = user.id.0.try_into() {
            db_handler.set_in_bot(user_id, true).await;

            if let Some(text) = msg.text() {
                db_handler
                    .add_edited_message_event(
                        user_id,
                        msg.date.into(),
                        msg.text().unwrap_or("").to_string(),
                    )
                    .await;

                match db_handler.get_exact(text).await {
                    Some(result) => {
                        let settings = db_handler.get_settings(user_id).await;
                        let deep_link_url = deep_link(me.username(), &result.query);

                        send_definition(
                            bot.clone(),
                            msg.chat.id,
                            &result,
                            &deep_link_url,
                            settings,
                            Some("😌 ¡Ahora sí!"),
                            Some(msg.id),
                        )
                        .await?;

                        db_handler
                            .add_sent_definition_event(user_id, msg.date.into(), result.query)
                            .await;
                    }
                    None => {
                        let url = match reqwest::Url::parse(&format!("https://dle.rae.es/{}", text))
                        {
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

                        let not_found = format!(include_str!("templates/not_found.txt"), text, "");

                        bot.send_message(msg.chat.id, format!("😐 Así tampoco\n\n{}", not_found))
                            .reply_markup(inline_keyboard)
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await?;
                    }
                }
            }
        }
    }
    Ok(())
}
