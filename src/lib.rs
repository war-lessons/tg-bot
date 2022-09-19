mod add;
mod config;
mod error;
mod lesson;
mod replier;
mod spam_token;
mod text;

pub use add::add_lesson;
pub use config::CONF;
pub use error::{eprint_error, internal_error, log_error, Error, Result};
pub use lesson::{LessonReadOptions, LessonStatusRange, SetLessonStatus};
pub use replier::{Replier, Reply, ReplyResult};
pub use spam_token::SpamTokenGenerator;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, MediaKind, MediaText, Message, MessageKind,
};
pub use text::{Lang, Translate, Translations, TEXT};

pub fn init_logging() -> Result<()> {
    if CONF.journal_logging {
        systemd_journal_logger::init().map_err(Error::InitSystemdLogging)?;
        log::set_max_level(log::LevelFilter::Warn);
    } else {
        pretty_env_logger::init();
    }
    Ok(())
}

pub async fn start_keyboard(
    pool: &sqlx::PgPool,
    lang: Lang,
    is_moderator: bool,
) -> InlineKeyboardMarkup {
    let mut lines = vec![vec![
        InlineKeyboardButton::callback(
            TEXT.read_best.to(lang),
            LessonReadOptions::new(LessonStatusRange::Best, None).to_command(),
        ),
        InlineKeyboardButton::callback(
            TEXT.read_approved.to(lang),
            LessonReadOptions::new(LessonStatusRange::Approved, None).to_command(),
        ),
        InlineKeyboardButton::callback(
            TEXT.read_all.to(lang),
            LessonReadOptions::new(LessonStatusRange::All, None).to_command(),
        ),
    ]];
    if is_moderator {
        let (new, rejected) = sqlx::query!(
            r#"
            SELECT
                (SELECT count(*) FROM lesson WHERE status = 'new') as "new!",
                (SELECT count(*) FROM lesson WHERE status = 'rejected') as "rejected!"
            "#
        )
        .map(|r| (r.new, r.rejected))
        .fetch_one(pool)
        .await
        .unwrap_or((-1, -1));
        lines.push(vec![
            InlineKeyboardButton::callback(
                format!("Moderate New ({new})"),
                LessonReadOptions::new(LessonStatusRange::New, None).to_command(),
            ),
            InlineKeyboardButton::callback(
                format!("Moderate Rejected ({rejected})"),
                LessonReadOptions::new(LessonStatusRange::Rejected, None).to_command(),
            ),
        ])
    };
    lines.push(vec![
        InlineKeyboardButton::callback(TEXT.add_lesson.to(lang), "/add"),
        InlineKeyboardButton::callback(TEXT.help.to(lang), "/help"),
    ]);
    InlineKeyboardMarkup::new(lines)
}

pub fn message_text(message: &Message) -> Option<&str> {
    match message.kind {
        MessageKind::Common(ref common) => match common.media_kind {
            MediaKind::Text(MediaText { ref text, .. }) => Some(text),
            _ => None,
        },
        _ => None,
    }
}
