use once_cell::sync::Lazy;
use serde::Deserialize;
use teloxide::types::{Message, MessageKind, User};

pub static TEXT: Lazy<Text> =
    Lazy::new(|| toml::from_str(include_str!("./text.toml")).expect("`TEXT` from yaml"));

#[derive(Clone, Copy, Debug, Default)]
pub enum Lang {
    #[default]
    En,
    Ru,
    Ua,
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
    en: String,
    ru: String,
    ua: String,
}

impl Translations {
    pub fn to(&self, lang: Lang) -> &str {
        match lang {
            Lang::En => &self.en,
            Lang::Ru => &self.ru,
            Lang::Ua => &self.ua,
        }
    }

    pub fn to_default(&self) -> &str {
        self.to(Lang::default())
    }
}

impl From<&str> for Lang {
    fn from(value: &str) -> Self {
        match value {
            "en" => Self::En,
            "ru" => Self::Ru,
            "uk" => Self::Ua,
            _ => Self::default(),
        }
    }
}

impl From<String> for Lang {
    fn from(value: String) -> Self {
        match value.as_str() {
            "en" => Self::En,
            "ru" => Self::Ru,
            "uk" => Self::Ua,
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

impl From<&User> for Lang {
    fn from(value: &User) -> Self {
        value
            .language_code
            .as_deref()
            .map(Into::into)
            .unwrap_or_default()
    }
}

impl From<&Message> for Lang {
    fn from(value: &Message) -> Self {
        if let MessageKind::Common(common) = &value.kind {
            common.from.as_ref().map(Into::into).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}
