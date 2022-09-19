use once_cell::sync::Lazy;
use serde::Deserialize;
use teloxide::types::{Message, MessageKind};

pub static TEXT: Lazy<Text> =
    Lazy::new(|| toml::from_str(include_str!("./text.toml")).expect("`TEXT` from yaml"));

#[derive(Clone, Copy, Debug, Default)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

#[derive(Deserialize)]
pub struct Text {
    pub add_lesson: Translations,
    pub add_lesson_message: Translations,
    pub flood: Translations,
    pub help: Translations,
    pub help_message: Translations,
    pub internal_error: Translations,
    pub lesson_not_found: Translations,
    pub lesson_saved: Translations,
    pub no_more_lessons: Translations,
    pub read_all: Translations,
    pub read_approved: Translations,
    pub read_best: Translations,
    pub next_lesson: Translations,
    pub text_only: Translations,
    pub unknown_command: Translations,
}

#[derive(Deserialize)]
pub struct Translations {
    ru: String,
    en: String,
}

impl Translations {
    pub fn to(&self, lang: Lang) -> &str {
        match lang {
            Lang::Ru => &self.ru,
            _ => &self.en,
        }
    }

    pub fn to_default(&self) -> &str {
        self.to(Lang::default())
    }
}

impl From<&str> for Lang {
    fn from(value: &str) -> Self {
        match value {
            "ru" => Self::Ru,
            _ => Self::En,
        }
    }
}

impl From<String> for Lang {
    fn from(value: String) -> Self {
        match value.as_str() {
            "en" => Self::En,
            "ru" => Self::Ru,
            _ => Self::default(),
        }
    }
}

pub trait Translate {
    fn translate(self, lang: Lang) -> String;
}

impl Translate for &'static Translations {
    fn translate(self, lang: Lang) -> String {
        self.to(lang).into()
    }
}

impl<T: Into<String>> Translate for T {
    fn translate(self, _lang: Lang) -> String {
        self.into()
    }
}

impl From<&Message> for Lang {
    fn from(value: &Message) -> Self {
        if let MessageKind::Common(common) = &value.kind {
            common
                .from
                .as_ref()
                .and_then(|user| user.language_code.as_deref())
                .map(Into::into)
                .unwrap_or_default()
        } else {
            Default::default()
        }
    }
}
