use serde::Serialize;
use teloxide::prelude::*;
use teloxide::requests::{JsonRequest, Payload};
use teloxide::types::{
    InlineQueryResultArticle, LinkPreviewOptions, Message, Recipient, ReplyParameters, True,
    WebAppInfo,
};

use crate::utils::DISABLED_LINK_PREVIEW;

/// Flat alternative to teloxide's `InlineQueryResultsButton` that avoids the
/// flatten+rename_all enum serialization issue with serde_with.
#[derive(Clone, Serialize)]
pub struct InlineQueryButton {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_parameter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_app: Option<WebAppInfo>,
}

impl InlineQueryButton {
    pub fn start_parameter(text: impl Into<String>, param: impl Into<String>) -> Self {
        Self { text: text.into(), start_parameter: Some(param.into()), web_app: None }
    }
}

#[derive(Clone, Serialize)]
pub struct InputRichMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_rtl: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_entity_detection: Option<bool>,
}

#[derive(Clone, Serialize)]
pub struct InputRichMessageContent {
    rich_message: InputRichMessage,
}

impl InputRichMessageContent {
    pub fn new(rich_message: InputRichMessage) -> Self {
        Self { rich_message }
    }
}

/// Flat alternative to teloxide's `InputMessageContent::Text`, so that inline
/// results can carry a classic HTML message for users who prefer it.
#[derive(Clone, Serialize)]
pub struct InputTextMessageContent {
    pub message_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,
}

impl InputTextMessageContent {
    pub fn html(message_text: impl Into<String>) -> Self {
        Self {
            message_text: message_text.into(),
            parse_mode: Some("HTML"),
            link_preview_options: Some(DISABLED_LINK_PREVIEW),
        }
    }
}

/// Content of an inline result: rich message or classic HTML one.
#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum InputMessageContent {
    Rich(InputRichMessageContent),
    Text(InputTextMessageContent),
}

impl From<InputRichMessageContent> for InputMessageContent {
    fn from(value: InputRichMessageContent) -> Self {
        Self::Rich(value)
    }
}

impl From<InputTextMessageContent> for InputMessageContent {
    fn from(value: InputTextMessageContent) -> Self {
        Self::Text(value)
    }
}

/// Minimal inline keyboard for the hand-rolled payloads below; teloxide's own
/// types hit the same flatten+rename_all serialization issue as the button above.
#[derive(Clone, Serialize)]
pub struct RichInlineKeyboardButton {
    pub text: String,
    pub callback_data: String,
}

impl RichInlineKeyboardButton {
    pub fn callback(text: impl Into<String>, callback_data: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            callback_data: callback_data.into(),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct RichInlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<RichInlineKeyboardButton>>,
}

impl RichInlineKeyboardMarkup {
    pub fn new(inline_keyboard: Vec<Vec<RichInlineKeyboardButton>>) -> Self {
        Self { inline_keyboard }
    }
}

#[derive(Clone, Serialize)]
pub struct RichInlineQueryResultArticle {
    #[serde(rename = "type")]
    result_type: &'static str,
    pub id: String,
    pub title: String,
    pub input_message_content: InputMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RichInlineQueryResultArticle {
    pub fn new<S1, S2>(
        id: S1,
        title: S2,
        input_message_content: impl Into<InputMessageContent>,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            result_type: "article",
            id: id.into(),
            title: title.into(),
            input_message_content: input_message_content.into(),
            description: None,
        }
    }

    pub fn description<S: Into<String>>(mut self, val: S) -> Self {
        self.description = Some(val.into());
        self
    }
}

pub trait InlineQueryResultArticleExt {
    fn new_with_rich_message<S1, S2>(
        id: S1,
        title: S2,
        content: impl Into<InputMessageContent>,
    ) -> RichInlineQueryResultArticle
    where
        S1: Into<String>,
        S2: Into<String>;
}

impl InlineQueryResultArticleExt for InlineQueryResultArticle {
    fn new_with_rich_message<S1, S2>(
        id: S1,
        title: S2,
        content: impl Into<InputMessageContent>,
    ) -> RichInlineQueryResultArticle
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        RichInlineQueryResultArticle::new(id, title, content)
    }
}

#[derive(Clone, Serialize)]
pub struct AnswerInlineQueryRich {
    pub inline_query_id: String,
    pub results: Vec<RichInlineQueryResultArticle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_personal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<InlineQueryButton>,
}

impl Payload for AnswerInlineQueryRich {
    type Output = True;
    const NAME: &'static str = "answerInlineQuery";
}

pub trait RichInlineQueryExt {
    fn answer_inline_query_rich(
        &self,
        inline_query_id: impl Into<String>,
        results: Vec<RichInlineQueryResultArticle>,
    ) -> JsonRequest<AnswerInlineQueryRich>;
}

impl RichInlineQueryExt for Bot {
    fn answer_inline_query_rich(
        &self,
        inline_query_id: impl Into<String>,
        results: Vec<RichInlineQueryResultArticle>,
    ) -> JsonRequest<AnswerInlineQueryRich> {
        JsonRequest::new(
            self.clone(),
            AnswerInlineQueryRich {
                inline_query_id: inline_query_id.into(),
                results,
                cache_time: None,
                is_personal: None,
                next_offset: None,
                button: None,
            },
        )
    }
}

#[derive(Clone, Serialize)]
pub struct SendRichMessage {
    pub chat_id: Recipient, // serializes as integer OR @username, like teloxide's own methods
    pub rich_message: InputRichMessage,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<RichInlineKeyboardMarkup>,
    // add the remaining optional fields the same way as you need them
}

// This is the part that wires it into teloxide.
impl Payload for SendRichMessage {
    type Output = Message; // the method returns the sent Message
    const NAME: &'static str = "sendRichMessage";
}

pub trait RichMessageExt {
    fn send_rich_message(
        &self,
        chat_id: impl Into<Recipient>,
        rich_message: InputRichMessage,
    ) -> JsonRequest<SendRichMessage>;
}

impl RichMessageExt for Bot {
    fn send_rich_message(
        &self,
        chat_id: impl Into<Recipient>,
        rich_message: InputRichMessage,
    ) -> JsonRequest<SendRichMessage> {
        JsonRequest::new(
            self.clone(),
            SendRichMessage {
                chat_id: chat_id.into(),
                rich_message,
                business_connection_id: None,
                message_thread_id: None,
                disable_notification: None,
                protect_content: None,
                reply_parameters: None,
                reply_markup: None,
            },
        )
    }
}
