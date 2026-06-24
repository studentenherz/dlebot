use serde::Serialize;
use teloxide::prelude::*;
use teloxide::requests::{JsonRequest, Payload};
use teloxide::types::{
    InlineQueryResultArticle, Message, Recipient, ReplyParameters, True, WebAppInfo,
};

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

#[derive(Clone, Serialize)]
pub struct RichInlineQueryResultArticle {
    #[serde(rename = "type")]
    result_type: &'static str,
    pub id: String,
    pub title: String,
    pub input_message_content: InputRichMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RichInlineQueryResultArticle {
    pub fn new<S1, S2>(id: S1, title: S2, input_message_content: InputRichMessageContent) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            result_type: "article",
            id: id.into(),
            title: title.into(),
            input_message_content,
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
        content: InputRichMessageContent,
    ) -> RichInlineQueryResultArticle
    where
        S1: Into<String>,
        S2: Into<String>;
}

impl InlineQueryResultArticleExt for InlineQueryResultArticle {
    fn new_with_rich_message<S1, S2>(
        id: S1,
        title: S2,
        content: InputRichMessageContent,
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
            },
        )
    }
}
