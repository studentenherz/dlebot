/// Split one string into multiple strings with a maximum length of `chars_per_string`.
/// Splits by '\n', '. ' or ' ' in this priority.
pub fn smart_split(text: &str, chars_per_string: usize) -> Vec<&str> {
    let mut result: Vec<&str> = vec![];
    let mut left: usize = 0;

    while text.len() - left >= chars_per_string {
        if let Some(pos) = text[left..=(left + chars_per_string)].rfind('\n') {
            result.push(&text[left..=(left + pos)]);
            left += pos + 1;
        } else if let Some(pos) = text[left..(left + chars_per_string)].rfind(". ") {
            result.push(&text[left..=(left + pos)]);
            left += pos + 1;
        } else if let Some(pos) = text[left..(left + chars_per_string)].rfind(' ') {
            result.push(&text[left..=(left + pos)]);
            left += pos + 1;
        } else {
            result.push(&text[left..=(left + chars_per_string)]);
            left += chars_per_string + 1;
        }
    }
    result.push(&text[left..]);

    result
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
