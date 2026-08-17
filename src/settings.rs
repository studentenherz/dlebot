use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::{
    rich_messages::{
        InputRichMessage, RichInlineKeyboardButton, RichInlineKeyboardMarkup, RichMessageExt,
    },
    utils::{deep_link, DISABLED_LINK_PREVIEW},
    DLEBot,
};

pub const FORMAT_RICH_CALLBACK_DATA: &str = "__fmt_rich";
pub const FORMAT_CLASSIC_CALLBACK_DATA: &str = "__fmt_classic";
pub const FORMAT_RICH_DEMO_CALLBACK_DATA: &str = "__fmt_rich_demo";

pub const START_SETTINGS: &str = "!ajustes";
pub const START_DEMO: &str = "!prueba";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MessageFormat {
    #[default]
    Rich,
    Classic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct UserSettings {
    pub format: MessageFormat,
}

impl MessageFormat {
    pub fn from_rich_text(rich_text: bool) -> Self {
        if rich_text {
            Self::Rich
        } else {
            Self::Classic
        }
    }

    pub fn is_rich(&self) -> bool {
        *self == Self::Rich
    }
}

impl UserSettings {
    pub fn is_rich(&self) -> bool {
        self.format.is_rich()
    }
}

pub fn settings_text(bot_username: &str, first_name: &str, settings: UserSettings) -> String {
    let (format_name, format_help) = match settings.format {
        MessageFormat::Rich => (
            "Enriquecido",
            "Las definiciones se envían con título, listas y sangrías. Necesita una \
             versión reciente de Telegram; si las ves mal formateadas, elige «Clásico».",
        ),
        MessageFormat::Classic => (
            "Clásico",
            "Las definiciones se envían solo con negritas, cursivas y enlaces. \
             Funciona en cualquier versión de Telegram.",
        ),
    };

    format!(
        include_str!("templates/settings.txt"),
        first_name,
        format_name,
        format_help,
        deep_link(bot_username, START_DEMO)
    )
}

pub fn settings_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("Enriquecido", FORMAT_RICH_CALLBACK_DATA),
        InlineKeyboardButton::callback("Clásico", FORMAT_CLASSIC_CALLBACK_DATA),
    ]])
}

fn demo_keyboard() -> RichInlineKeyboardMarkup {
    RichInlineKeyboardMarkup::new(vec![vec![RichInlineKeyboardButton::callback(
        "Cambiar a modo enriquecido",
        FORMAT_RICH_DEMO_CALLBACK_DATA,
    )]])
}

pub async fn send_settings(
    bot_username: &str,
    bot: &DLEBot,
    chat_id: ChatId,
    first_name: &str,
    settings: UserSettings,
) -> ResponseResult<()> {
    bot.send_message(chat_id, settings_text(bot_username, first_name, settings))
        .parse_mode(ParseMode::Html)
        .link_preview_options(DISABLED_LINK_PREVIEW)
        .reply_markup(settings_keyboard())
        .await?;

    Ok(())
}

pub async fn send_format_demo(bot: &DLEBot, chat_id: ChatId) -> ResponseResult<()> {
    let html = include_str!("templates/settings_demo.html");

    let mut request = bot.send_rich_message(
        chat_id,
        InputRichMessage {
            html: Some(html.to_string()),
            markdown: None,
            is_rtl: None,
            skip_entity_detection: None,
        },
    );
    request.reply_markup = Some(demo_keyboard());

    if let Err(error) = request.await {
        log::warn!("Failed to send the rich text sample: {:?}", error);

        bot.send_message(chat_id, include_str!("templates/settings_demo_failed.txt"))
            .parse_mode(ParseMode::Html)
            .link_preview_options(DISABLED_LINK_PREVIEW)
            .reply_markup(settings_keyboard())
            .await?;
    }

    Ok(())
}
