use crate::{lesson::LessonStatus, Translations, TEXT};
use std::fmt::Write;

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum Error {
    /// Lesson::get({1:?}, {2:?})
    ReadNextLesson(#[source] sqlx::Error, LessonStatus, Option<i32>),
    /// LessonModeration::get({1})
    GetLessonModeration(#[source] sqlx::Error, i32),
    /// SetLessonStatus::set({1}, {2:?})
    SetLessonStatus(#[source] sqlx::Error, i32, LessonStatus),
    /// LessonModeration::next
    NextLessonModeration(#[source] sqlx::Error),
    /// Systemd logging init
    InitSystemdLogging(#[source] log::SetLoggerError),
    /// Create pg pool
    CreatePgPool(#[source] sqlx::Error),
    /// Migrate
    Migrate(#[source] sqlx::migrate::MigrateError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Logs an error and returns a bot reply
pub fn internal_error(e: &impl std::error::Error) -> &'static Translations {
    log_error(e);
    &TEXT.internal_error
}

pub fn log_error(e: &impl std::error::Error) {
    log::error!("{}", error_chain(e));
}

pub fn eprint_error(e: &impl std::error::Error) {
    eprintln!("{}", error_chain(e));
}

fn error_chain(e: &impl std::error::Error) -> String {
    let mut s = e.to_string();
    let mut src = e.source();
    if src.is_some() {
        s.push_str("\nCaused by:");
    }
    while let Some(cause) = src {
        write!(s, "\n  {}", cause).ok();
        src = cause.source();
    }
    s
}
