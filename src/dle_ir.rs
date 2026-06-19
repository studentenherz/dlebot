use std::collections::HashSet;
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
            "tg-spoiler",
            "tg-emoji",
            "tg-time",
            "code",
            "pre",
            "blockquote",
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

pub struct DleWord {
    /// The query string used to look up this word (URL slug).
    pub query: String,
    /// The headword actually returned; differs from `query` for inflected lookups.
    pub resolved_headword: Option<String>,
    pub entries: Vec<DleEntry>,
}

pub struct DleEntry {
    pub headword: String,
    pub homograph: Option<i16>,
    pub etymology_text: Option<String>,
    pub etymology_html: Option<String>,
    pub sense_groups: Vec<DleSenseGroup>,
}

pub enum DleSenseGroup {
    Main {
        senses: Vec<DleSense>,
    },
    ComplexForm {
        form_text: Option<String>,
        senses: Vec<DleSense>,
    },
}

pub struct DleSense {
    pub number: Option<i16>,
    pub definition_text: Option<String>,
    pub definition_html: Option<String>,
    pub labels: Vec<DleLabel>,
    pub examples: Vec<DleExample>,
    pub relations: Vec<DleRelation>,
}

pub struct DleRelation {
    pub kind: String, // "synonym" | "antonym"
    pub word: String,
    pub scope: Option<String>,
}

pub struct DleExample {
    pub text: String,
}

pub struct DleLabel {
    pub abbr: String,
    pub full_text: Option<String>,
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

    pub fn to_html(&self) -> String {
        let mut out = String::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            entry.write_html(&mut out);
        }
        out.trim_end().to_string()
    }

    pub fn to_html_with_deeplink(&self, url: &str) -> String {
        let html = self.to_html();
        let headword = self.headword();
        let linked = format!("<b><a href=\"{}\">{}</a></b>", url, headword);
        html.replacen(&format!("<b>{}</b>", headword), &linked, 1)
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
    pub(crate) fn write_html(&self, out: &mut String) {
        let sup = self.homograph.map(superscript).unwrap_or("");
        let _ = write!(out, "<b>{}</b>{}", self.headword, sup);

        let etym = self
            .etymology_html
            .as_deref()
            .or(self.etymology_text.as_deref())
            .unwrap_or("")
            .trim();
        if !etym.is_empty() {
            let _ = write!(out, "\n<i>({})</i>", sanitize_html(etym));
        }
        out.push('\n');

        for group in &self.sense_groups {
            group.write_html(out);
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
            let _ = write!(out, "\n<b>{}</b>\n", form);
        }
        for sense in self.senses() {
            sense.write_html(out);
        }
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

impl DleSense {
    fn write_html(&self, out: &mut String) {
        if let Some(num) = self.number {
            let _ = write!(out, "{}. ", num);
        }

        if !self.labels.is_empty() {
            let labels: Vec<&str> = self.labels.iter().map(|l| l.abbr.as_str()).collect();
            let _ = write!(out, "{} ", labels.join(" "));
        }

        let def = self.definition_text.as_deref().unwrap_or("").trim();
        out.push_str(def);
        out.push('\n');

        for ex in &self.examples {
            let _ = write!(out, "▸ <i>{}</i>\n", ex.text.trim());
        }

        let synonyms: Vec<&str> = self
            .relations
            .iter()
            .filter(|r| r.kind == "synonym")
            .map(|r| r.word.as_str())
            .collect();
        if !synonyms.is_empty() {
            let _ = write!(out, "<i>Sin.:</i> {}\n", synonyms.join(", "));
        }

        let antonyms: Vec<&str> = self
            .relations
            .iter()
            .filter(|r| r.kind == "antonym")
            .map(|r| r.word.as_str())
            .collect();
        if !antonyms.is_empty() {
            let _ = write!(out, "<i>Ant.:</i> {}\n", antonyms.join(", "));
        }
    }

    fn write_text(&self, out: &mut String) {
        if let Some(num) = self.number {
            let _ = write!(out, "{}. ", num);
        }

        if !self.labels.is_empty() {
            let labels: Vec<&str> = self.labels.iter().map(|l| l.abbr.as_str()).collect();
            let _ = write!(out, "{} ", labels.join(" "));
        }

        let def = self.definition_text.as_deref().unwrap_or("").trim();
        out.push_str(def);
        out.push('\n');

        for ex in &self.examples {
            let _ = write!(out, "▸ {}\n", ex.text.trim());
        }

        let synonyms: Vec<&str> = self
            .relations
            .iter()
            .filter(|r| r.kind == "synonym")
            .map(|r| r.word.as_str())
            .collect();
        if !synonyms.is_empty() {
            let _ = write!(out, "Sin.: {}\n", synonyms.join(", "));
        }

        let antonyms: Vec<&str> = self
            .relations
            .iter()
            .filter(|r| r.kind == "antonym")
            .map(|r| r.word.as_str())
            .collect();
        if !antonyms.is_empty() {
            let _ = write!(out, "Ant.: {}\n", antonyms.join(", "));
        }
    }
}
