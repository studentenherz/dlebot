use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Me},
    utils::command::BotCommands,
};

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

pub async fn handle_messages(bot: Bot, msg: Message, me: Me) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Start) => {
                bot.parse_mode(teloxide::types::ParseMode::Html)
                    .send_message(msg.chat.id, include_str!("templates/start.txt"))
                    .disable_web_page_preview(true)
                    .await?
            }
            Ok(Command::Help | Command::Ayuda) => {
                let keyboard = InlineKeyboardMarkup::new(vec![vec![
                    InlineKeyboardButton::switch_inline_query("Buscar definición", ""),
                ]]);

                bot.parse_mode(teloxide::types::ParseMode::Html)
                    .send_message(
                        msg.chat.id,
                        format!(
                            include_str!("templates/help.txt"),
                            bot_username = me.username()
                        ),
                    )
                    .reply_markup(keyboard)
                    .await?
            }
            Ok(_) => {
                bot.send_message(msg.chat.id, "Not implemented command")
                    .await?
            }
            Err(_) => bot.send_message(msg.chat.id, "Just text").await?,
        };
    }
    Ok(())
}
