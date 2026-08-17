use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::sync::OnceLock;

use regex::Regex;

fn to_superscript_char(c: char) -> char {
    match c {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' => '⁻',
        '=' => '⁼',
        '(' => '⁽',
        ')' => '⁾',
        'a' => 'ᵃ',
        'b' => 'ᵇ',
        'c' => 'ᶜ',
        'd' => 'ᵈ',
        'e' => 'ᵉ',
        'f' => 'ᶠ',
        'g' => 'ᵍ',
        'h' => 'ʰ',
        'i' => 'ⁱ',
        'j' => 'ʲ',
        'k' => 'ᵏ',
        'l' => 'ˡ',
        'm' => 'ᵐ',
        'n' => 'ⁿ',
        'o' => 'ᵒ',
        'p' => 'ᵖ',
        'r' => 'ʳ',
        's' => 'ˢ',
        't' => 'ᵗ',
        'u' => 'ᵘ',
        'v' => 'ᵛ',
        'w' => 'ʷ',
        'x' => 'ˣ',
        'y' => 'ʸ',
        'z' => 'ᶻ',
        _ => c,
    }
}

pub(crate) fn sanitize_html(html: &str) -> String {
    static SUP_RE: OnceLock<Regex> = OnceLock::new();
    static TAG_RE: OnceLock<Regex> = OnceLock::new();
    static SUPPORTED: OnceLock<HashSet<&'static str>> = OnceLock::new();

    let sup_re = SUP_RE.get_or_init(|| Regex::new(r"(?i)<sup>(.*?)</sup>").unwrap());
    let html = sup_re.replace_all(html, |caps: &regex::Captures| {
        caps[1].chars().map(to_superscript_char).collect::<String>()
    });

    let re = TAG_RE.get_or_init(|| Regex::new(r"</?([a-zA-Z][a-zA-Z0-9-]*)[^>]*>").unwrap());
    let supported = SUPPORTED.get_or_init(|| {
        [
            // inline formatting
            "a",
            "b",
            "strong",
            "i",
            "em",
            "u",
            "ins",
            "s",
            "strike",
            "del",
            "code",
            "mark",
            "sub",
            // telegram-specific inline
            "tg-spoiler",
            "tg-emoji",
            "tg-time",
            "tg-reference",
            // block / structure
            "pre",
            "blockquote",
            "aside",
            "cite",
            "p",
            "br",
            "hr",
            "footer",
            // headings
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            // lists
            "ul",
            "ol",
            "li",
            "input",
            // media
            "img",
            "video",
            "audio",
            "figure",
            "figcaption",
            // telegram media / layout
            "tg-map",
            "tg-collage",
            "tg-slideshow",
            // table
            "table",
            "caption",
            "tr",
            "th",
            "td",
            // collapsible
            "details",
            "summary",
            // math
            "tg-math",
            "tg-math-block",
        ]
        .into_iter()
        .collect()
    });

    re.replace_all(html.as_ref(), |caps: &regex::Captures| {
        if supported.contains(caps[1].to_lowercase().as_str()) {
            caps[0].to_string()
        } else {
            String::new()
        }
    })
    .into_owned()
}

/// Sanitize an HTML fragment for classic `parse_mode=HTML` messages, which
/// accept a much smaller set of tags than rich messages do.
pub(crate) fn sanitize_classic_html(html: &str) -> String {
    static SUP_RE: OnceLock<Regex> = OnceLock::new();
    static TAG_RE: OnceLock<Regex> = OnceLock::new();
    static SUPPORTED: OnceLock<HashSet<&'static str>> = OnceLock::new();
    static REPLACE_BY_TAGS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

    let sup_re = SUP_RE.get_or_init(|| Regex::new(r"(?i)<sup>(.*?)</sup>").unwrap());
    let html = sup_re.replace_all(html, |caps: &regex::Captures| {
        caps[1].chars().map(to_superscript_char).collect::<String>()
    });

    let re = TAG_RE.get_or_init(|| Regex::new(r"(</?)([a-zA-Z][a-zA-Z0-9-]*)[^>]*>").unwrap());
    let supported = SUPPORTED.get_or_init(|| {
        [
            "a",
            "b",
            "strong",
            "i",
            "em",
            "u",
            "ins",
            "s",
            "strike",
            "del",
            "code",
            "pre",
            "blockquote",
            "tg-spoiler",
        ]
        .into_iter()
        .collect()
    });
    let replace_by_tags = REPLACE_BY_TAGS.get_or_init(|| [("abbr", "b")].into_iter().collect());

    re.replace_all(html.as_ref(), |caps: &regex::Captures| {
        let tag = caps[2].to_lowercase();
        if supported.contains(tag.as_str()) {
            caps[0].to_string()
        } else if let Some(replacement) = replace_by_tags.get(tag.as_str()) {
            format!("{}{}>", &caps[1], replacement)
        } else {
            String::new()
        }
    })
    .into_owned()
}

/// Escape plain text that is about to be embedded in classic HTML.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Wrap in `<i>` unless the fragment already carries italics, since Telegram
/// rejects identically nested entities.
fn italicize(html: &str) -> String {
    if html.contains("<i>") || html.contains("<em>") {
        html.to_string()
    } else {
        format!("<i>{}</i>", html)
    }
}

#[derive(Debug)]
pub struct DleWord {
    /// The query string used to look up this word (URL slug).
    pub query: String,
    /// The headword actually returned; differs from `query` for inflected lookups.
    pub resolved_headword: Option<String>,
    pub entries: Vec<DleEntry>,
}

#[derive(Debug)]
pub struct DleEntry {
    pub headword: String,
    pub homograph: Option<i16>,
    pub etymology_text: Option<String>,
    pub etymology_html: Option<String>,
    pub sense_groups: Vec<DleSenseGroup>,
}

#[derive(Debug)]
pub enum DleSenseGroup {
    Main {
        senses: Vec<DleSense>,
    },
    ComplexForm {
        form_text: Option<String>,
        senses: Vec<DleSense>,
    },
}

#[derive(Debug)]
pub struct DleSense {
    pub number: Option<i16>,
    pub definition_text: Option<String>,
    pub definition_html: Option<String>,
    pub examples: Vec<DleExample>,
    pub relations: Vec<DleRelation>,
}

#[derive(Debug)]
pub struct DleRelation {
    pub kind: String, // "synonym" | "antonym"
    pub word: String,
    pub homograph: Option<i16>,
    #[allow(unused)]
    pub scope: Option<String>,
}

#[derive(Debug)]
pub struct DleExample {
    pub text: String,
}

fn superscript(n: i16) -> &'static str {
    match n {
        1 => "¹",
        2 => "²",
        3 => "³",
        4 => "⁴",
        5 => "⁵",
        6 => "⁶",
        7 => "⁷",
        8 => "⁸",
        9 => "⁹",
        _ => "",
    }
}

impl DleWord {
    pub fn headword(&self) -> &str {
        self.resolved_headword.as_deref().unwrap_or(&self.query)
    }

    pub fn to_html(&self, url: Option<&str>) -> String {
        let mut out = String::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            entry.write_html(&mut out, url);
        }
        out.trim_end().to_string()
    }

    pub fn to_html_with_deeplink(&self, url: &str) -> String {
        self.to_html(Some(url))
    }

    /// Render for `sendMessage` with `parse_mode=HTML`: same content as
    /// [`Self::to_html`], but using only the tags the Bot API supports.
    pub fn to_classic_html(&self, url: Option<&str>) -> String {
        let mut out = String::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            entry.write_classic_html(&mut out, url);
        }
        out.trim_end().to_string()
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            entry.write_text(&mut out);
        }
        out.trim_end().to_string()
    }
}

impl DleEntry {
    pub(crate) fn write_html(&self, out: &mut String, url: Option<&str>) {
        match (url, self.homograph) {
            (Some(u), Some(n)) => {
                let _ = write!(
                    out,
                    "<h1><a href=\"{}\">{}</a><sup>{}</sup></h1>",
                    u, self.headword, n
                );
            }
            (Some(u), None) => {
                let _ = write!(out, "<h1><a href=\"{}\">{}</a></h1>", u, self.headword);
            }
            (None, Some(n)) => {
                let _ = write!(out, "<h1>{}<sup>{}</sup></h1>", self.headword, n);
            }
            (None, None) => {
                let _ = write!(out, "<h1>{}</h1>", self.headword);
            }
        }

        let etym = self
            .etymology_html
            .as_deref()
            .or(self.etymology_text.as_deref())
            .unwrap_or("")
            .trim();
        if !etym.is_empty() {
            let _ = write!(out, "<p>{}</p>", sanitize_html(etym));
        }

        for group in &self.sense_groups {
            group.write_html(out);
        }
    }

    pub(crate) fn write_classic_html(&self, out: &mut String, url: Option<&str>) {
        let escaped_headword = escape_html(&self.headword);
        let headword = match url {
            Some(u) => format!("<a href=\"{}\">{}</a>", u, escaped_headword),
            None => escaped_headword,
        };
        let sup = self.homograph.map(superscript).unwrap_or("");
        let _ = writeln!(out, "<b>{}</b>{}", headword, sup);

        let etym = match self.etymology_html.as_deref() {
            Some(html) => sanitize_classic_html(html),
            None => escape_html(self.etymology_text.as_deref().unwrap_or("")),
        };
        let etym = etym.trim();
        if !etym.is_empty() {
            let _ = writeln!(out, "{}", italicize(etym));
        }
        out.push('\n');

        for group in &self.sense_groups {
            group.write_classic_html(out);
        }
    }

    pub(crate) fn write_text(&self, out: &mut String) {
        let sup = self.homograph.map(superscript).unwrap_or("");
        let _ = write!(out, "{}{}", self.headword, sup);

        let etym = self.etymology_text.as_deref().unwrap_or("").trim();
        if !etym.is_empty() {
            let _ = write!(out, "\n({})", etym);
        }
        out.push('\n');

        for group in &self.sense_groups {
            group.write_text(out);
        }
    }
}

impl DleSenseGroup {
    fn senses(&self) -> &[DleSense] {
        match self {
            Self::Main { senses } | Self::ComplexForm { senses, .. } => senses,
        }
    }

    fn write_html(&self, out: &mut String) {
        if let Self::ComplexForm {
            form_text: Some(form),
            ..
        } = self
        {
            let _ = write!(out, "<p><b>{}</b></p>", form);
        }
        out.push_str("<ol>");
        for sense in self.senses() {
            sense.write_html(out);
        }
        out.push_str("</ol>");
    }

    fn write_classic_html(&self, out: &mut String) {
        if let Self::ComplexForm {
            form_text: Some(form),
            ..
        } = self
        {
            let _ = writeln!(out, "<b>{}</b>", escape_html(form));
        }
        for sense in self.senses() {
            sense.write_classic_html(out);
        }
        out.push('\n');
    }

    fn write_text(&self, out: &mut String) {
        if let Self::ComplexForm {
            form_text: Some(form),
            ..
        } = self
        {
            let _ = write!(out, "\n{}\n", form);
        }
        for sense in self.senses() {
            sense.write_text(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word() -> DleWord {
        DleWord {
            query: "prueba".to_string(),
            resolved_headword: None,
            entries: vec![DleEntry {
                headword: "prueba".to_string(),
                homograph: Some(1),
                etymology_text: Some("Del lat. proba & probāre.".to_string()),
                etymology_html: None,
                sense_groups: vec![DleSenseGroup::Main {
                    senses: vec![DleSense {
                        number: Some(1),
                        definition_text: Some("Acción de probar.".to_string()),
                        definition_html: Some(
                            "<abbr title=\"femenino\">f.</abbr> Acción de probar<sup>2</sup>."
                                .to_string(),
                        ),
                        examples: vec![DleExample {
                            text: "A > B".to_string(),
                        }],
                        relations: vec![DleRelation {
                            kind: "synonym".to_string(),
                            word: "ensayo".to_string(),
                            homograph: Some(2),
                            scope: None,
                        }],
                    }],
                }],
            }],
        }
    }

    #[test]
    fn classic_html_uses_only_supported_tags() {
        let html = word().to_classic_html(Some("https://t.me/bot?start=cHJ1ZWJh"));

        assert_eq!(
            html,
            "<b><a href=\"https://t.me/bot?start=cHJ1ZWJh\">prueba</a></b>¹\n\
             <i>Del lat. proba &amp; probāre.</i>\n\
             \n\
             <b>1.</b> <b>f.</b> Acción de probar².\n\
             ▸ <i>A &gt; B</i>\n\
             <i>Sin.:</i> ensayo²"
        );
    }

    #[test]
    fn classic_html_falls_back_to_plain_text_escaped() {
        let mut word = word();
        word.entries[0].sense_groups = vec![DleSenseGroup::Main {
            senses: vec![DleSense {
                number: None,
                definition_text: Some("Uno < dos".to_string()),
                definition_html: None,
                examples: vec![],
                relations: vec![],
            }],
        }];

        assert!(word.to_classic_html(None).contains("Uno &lt; dos"));
    }
}

impl DleSense {
    pub fn sanitized_definition_html(&self) -> Option<String> {
        static TAG_RE: OnceLock<Regex> = OnceLock::new();
        static TEXT_FORMATTING_TAGS: OnceLock<HashSet<&'static str>> = OnceLock::new();
        static REPLACE_BY_TAGS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

        let re = TAG_RE.get_or_init(|| Regex::new(r"(</?)([a-zA-Z][a-zA-Z0-9-]*)[^>]*>").unwrap());
        let allowed = TEXT_FORMATTING_TAGS.get_or_init(|| {
            [
                "b", "strong", "i", "em", "u", "ins", "s", "strike", "del", "mark", "sub", "sup",
            ]
            .into_iter()
            .collect()
        });
        let replace_by_tags = REPLACE_BY_TAGS.get_or_init(|| [("abbr", "b")].into_iter().collect());

        self.definition_html.as_deref().map(|html| {
            re.replace_all(html, |caps: &regex::Captures| {
                let caputured_tag = caps[2].to_lowercase();
                if allowed.contains(caputured_tag.as_str()) {
                    caps[0].to_string()
                } else if let Some(replacement_tag) = replace_by_tags.get(caputured_tag.as_str()) {
                    format!("{}{}>", &caps[1], replacement_tag)
                } else {
                    String::new()
                }
            })
            .into_owned()
        })
    }

    fn write_html(&self, out: &mut String) {
        match self.number {
            Some(n) => {
                let _ = write!(out, "<li value=\"{}\">", n);
            }
            None => out.push_str("<li>"),
        }

        let sanitized = self.sanitized_definition_html();
        let def = sanitized
            .as_deref()
            .or(self.definition_text.as_deref())
            .unwrap_or("")
            .trim();
        out.push_str(def);

        if !self.examples.is_empty() {
            out.push_str("<ul>");
            for ex in &self.examples {
                let _ = write!(out, "<li><i>{}</i></li>", ex.text.trim());
            }
            out.push_str("</ul>");
        }

        let fmt_html = |r: &DleRelation| match r.homograph {
            Some(n) => format!("{}<sup>{}</sup>", r.word, n),
            None => r.word.clone(),
        };

        let synonyms: Vec<String> = self
            .relations
            .iter()
            .filter(|r| r.kind == "synonym")
            .map(fmt_html)
            .collect();
        if !synonyms.is_empty() {
            let _ = write!(out, "<p><i>Sin.:</i> {}</p>", synonyms.join(", "));
        }

        let antonyms: Vec<String> = self
            .relations
            .iter()
            .filter(|r| r.kind == "antonym")
            .map(fmt_html)
            .collect();
        if !antonyms.is_empty() {
            let _ = write!(out, "<p><i>Ant.:</i> {}</p>", antonyms.join(", "));
        }

        out.push_str("</li>");
    }

    fn write_classic_html(&self, out: &mut String) {
        if let Some(num) = self.number {
            let _ = write!(out, "<b>{}.</b> ", num);
        }

        let def = match self.definition_html.as_deref() {
            Some(html) => sanitize_classic_html(html),
            None => escape_html(self.definition_text.as_deref().unwrap_or("")),
        };
        let _ = writeln!(out, "{}", def.trim());

        for ex in &self.examples {
            let _ = writeln!(out, "▸ <i>{}</i>", escape_html(ex.text.trim()));
        }

        let fmt_relation = |r: &DleRelation| match r.homograph {
            Some(n) => format!("{}{}", escape_html(&r.word), superscript(n)),
            None => escape_html(&r.word),
        };

        let synonyms: Vec<String> = self
            .relations
            .iter()
            .filter(|r| r.kind == "synonym")
            .map(fmt_relation)
            .collect();
        if !synonyms.is_empty() {
            let _ = writeln!(out, "<i>Sin.:</i> {}", synonyms.join(", "));
        }

        let antonyms: Vec<String> = self
            .relations
            .iter()
            .filter(|r| r.kind == "antonym")
            .map(fmt_relation)
            .collect();
        if !antonyms.is_empty() {
            let _ = writeln!(out, "<i>Ant.:</i> {}", antonyms.join(", "));
        }
    }

    fn write_text(&self, out: &mut String) {
        if let Some(num) = self.number {
            let _ = write!(out, "{}. ", num);
        }

        let def = self.definition_text.as_deref().unwrap_or("").trim();
        out.push_str(def);
        out.push('\n');

        for ex in &self.examples {
            let _ = writeln!(out, "▸ {}", ex.text.trim());
        }

        let fmt_text = |r: &DleRelation| match r.homograph {
            Some(n) => format!("{}{}", r.word, superscript(n)),
            None => r.word.clone(),
        };

        let synonyms: Vec<String> = self
            .relations
            .iter()
            .filter(|r| r.kind == "synonym")
            .map(fmt_text)
            .collect();
        if !synonyms.is_empty() {
            let _ = writeln!(out, "Sin.: {}", synonyms.join(", "));
        }

        let antonyms: Vec<String> = self
            .relations
            .iter()
            .filter(|r| r.kind == "antonym")
            .map(fmt_text)
            .collect();
        if !antonyms.is_empty() {
            let _ = writeln!(out, "Ant.: {}", antonyms.join(", "));
        }
    }
}
