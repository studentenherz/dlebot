use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};

pub const MAX_MASSAGE_LENGTH: usize = 4096;
pub const MAX_WOTD_LENGTH: usize = 400;
pub const SUBS_CALLBACK_DATA: &str = "__subs";
pub const DESUBS_CALLBACK_DATA: &str = "__desubs";
const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// Split one string into multiple strings with a maximum length of `chars_per_string`.
/// Splits by '\n', '. ' or ' ' in this priority.
pub fn smart_split(text: &str, chars_per_string: usize) -> Vec<&str> {
    let mut result: Vec<&str> = vec![];
    let mut left: usize = 0;

    while text.len() - left >= chars_per_string {
        if let Some(pos) = text[left..(left + chars_per_string)].rfind('\n') {
            result.push(&text[left..=(left + pos)]);
            left += pos + 1;
        } else if let Some(pos) = text[left..(left + chars_per_string)].rfind(". ") {
            result.push(&text[left..=(left + pos)]);
            left += pos + 1;
        } else if let Some(pos) = text[left..(left + chars_per_string)].rfind(' ') {
            result.push(&text[left..=(left + pos)]);
            left += pos + 1;
        } else {
            result.push(&text[left..(left + chars_per_string)]);
            left += chars_per_string;
        }
    }
    result.push(&text[left..]);

    result
}

/// Split one string into multiple strings with a maximum length of `chars_per_string`.
/// Splits ' '
pub fn split_by_whitespace(text: &str, chars_per_string: usize) -> Vec<&str> {
    let mut result: Vec<&str> = vec![];
    let mut left: usize = 0;
    let mut last_space_pos: usize = 0;
    let mut count: usize = 0;

    let mut open = false;
    for (i, c) in text.char_indices() {
        if c == '<' {
            open = true;
        }

        if !open {
            count += 1;

            if count > chars_per_string {
                result.push(&text[left..=last_space_pos]);
                left = last_space_pos + 1;
                count = i - last_space_pos;
            }

            if c == ' ' {
                last_space_pos = i;
            }
        }

        if c == '>' {
            open = false;
        }
    }
    result.push(&text[left..]);

    result
}

pub fn base64_encode(text: String) -> String {
    CUSTOM_ENGINE.encode(text)
}

pub fn base64_decode(text: String) -> Result<String, &'static str> {
    if let Ok(decoded_vec) = CUSTOM_ENGINE.decode(text) {
        if let Ok(decoded) = String::from_utf8(decoded_vec) {
            return Ok(decoded);
        }
    }

    Err("Error decoding")
}

#[test]
fn test_smart_split() {
    let text = "This is a normal string. \
    With some lines too long. And other no so, now I'm going to keep writing because this is important.";
    assert_eq!(
        smart_split(text, 50),
        [
            "This is a normal string.",
            " With some lines too long.",
            " And other no so, now I'm going to keep writing ",
            "because this is important."
        ]
    );
}
