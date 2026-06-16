use serde::Serialize;
use teloxide::prelude::*;
use teloxide::requests::{JsonRequest, Payload};
use teloxide::types::{Message, Recipient, ReplyParameters};

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
