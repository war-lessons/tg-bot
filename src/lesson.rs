use crate::{internal_error, start_keyboard, Error, Lang, Replier, ReplyResult, TEXT};
use sqlx::{query_as, PgPool};
use std::{convert::AsRef, str::FromStr};
use strum_macros::{AsRefStr, EnumString};
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use time::OffsetDateTime;

const VIEW_CMD: &str = "/view";
const SET_STATUS_CMD: &str = "/set-lesson-status";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct LessonReadOptions {
    status_range: LessonStatusRange,
    prev_lesson: Option<i32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum LessonStatusRange {
    Rejected,
    New,
    #[default]
    Approved,
    Best,
    All,
}

pub struct Lesson {
    id: i32,
    text: String,
    status: LessonStatus,
    created_at: OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumString, AsRefStr, sqlx::Type)]
#[sqlx(type_name = "lesson_status", rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum LessonStatus {
    Rejected,
    New,
    #[default]
    Approved,
    Best,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetLessonStatus {
    /// Kind of lessons we're moderating
    status_range: LessonStatusRange,
    lesson_id: i32,
    status: LessonStatus,
}

impl LessonReadOptions {
    pub fn new(status_range: LessonStatusRange, prev_lesson: Option<i32>) -> Self {
        Self {
            status_range,
            prev_lesson,
        }
    }

    pub fn from_command(cmd: &str) -> Option<Self> {
        if cmd.starts_with(VIEW_CMD) {
            let mut parts = cmd.split_whitespace();
            parts.next();
            let status_range = if let Some(s) = parts.next() {
                LessonStatusRange::from_str(s).ok()?
            } else {
                Default::default()
            };
            let prev_lesson = if let Some(s) = parts.next() {
                Some(s.parse().ok()?)
            } else {
                Default::default()
            };
            Some(Self {
                status_range,
                prev_lesson,
            })
        } else {
            None
        }
    }

    pub fn to_command(&self) -> String {
        if let Some(prev) = self.prev_lesson {
            format!("{VIEW_CMD} {} {prev}", self.status_range.as_ref())
        } else {
            format!("{VIEW_CMD} {}", self.status_range.as_ref())
        }
    }

    pub fn prev_lesson(&mut self, id: impl Into<i32>) -> &Self {
        self.prev_lesson = Some(id.into());
        self
    }

    pub async fn reply(&self, pool: &PgPool, repl: &Replier) -> ReplyResult {
        match Lesson::get(pool, self.status_range, self.prev_lesson).await {
            Ok(Some(lesson)) => {
                repl.send_text(lesson.message(repl.is_moderator()))
                    .reply_markup(lesson.keyboard(
                        self.status_range,
                        repl.lang,
                        repl.is_moderator(),
                    ))
                    .await?
            }
            Ok(None) => {
                repl.send_text(TEXT.no_more_lessons.to(repl.lang))
                    .reply_markup(start_keyboard(pool, repl.lang, repl.is_moderator()).await)
                    .await?
            }
            Err(e) => repl.send_text(internal_error(&e)).await?,
        };
        Ok(())
    }
}

impl SetLessonStatus {
    fn new(status_range: LessonStatusRange, lesson_id: i32, status: LessonStatus) -> Self {
        Self {
            status_range,
            lesson_id,
            status,
        }
    }

    pub fn from_command(cmd: &str) -> Option<Self> {
        if cmd.starts_with(SET_STATUS_CMD) {
            let mut parts = cmd.split_whitespace();
            parts.next();
            let status_range = parts
                .next()
                .and_then(|s| LessonStatusRange::from_str(s).ok())?;
            let lesson_id = parts.next().and_then(|s| s.parse().ok())?;
            let status = parts.next().and_then(|s| LessonStatus::from_str(s).ok())?;
            Some(Self {
                status_range,
                lesson_id,
                status,
            })
        } else {
            None
        }
    }

    fn to_command(&self) -> String {
        format!(
            "{} {} {} {}",
            SET_STATUS_CMD,
            self.status_range.as_ref(),
            self.lesson_id,
            self.status.as_ref()
        )
    }

    pub async fn reply(&self, pool: &PgPool, repl: &Replier) -> ReplyResult {
        match Lesson::set_status(pool, self.lesson_id, self.status).await {
            Ok(lesson) => {
                repl.edit_text(lesson.message(repl.is_moderator()))
                    .reply_markup(lesson.keyboard(
                        self.status_range,
                        repl.lang,
                        repl.is_moderator(),
                    ))
                    .await?
            }
            Err(e) => repl.send_text(internal_error(&e)).await?,
        };
        Ok(())
    }
}

impl Lesson {
    /// Returns a lesson to read after the `prev` lesson with a minimal status `min_status`
    async fn get(
        pool: &PgPool,
        status_range: LessonStatusRange,
        prev: Option<i32>,
    ) -> Result<Option<Self>, Error> {
        let (min_status, max_status) = status_range.range();
        query_as!(
            Self,
            r#"
            SELECT 
                id,
                text,
                status as "status: _",
                created_at
            FROM lesson 
            WHERE status >= $1
              AND status <= $2
              AND ($3::int IS NULL OR id < $3)
            ORDER BY id DESC
            LIMIT 1
            "#,
            min_status as LessonStatus,
            max_status as LessonStatus,
            prev,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| Error::ReadNextLesson(e, min_status, prev))
    }

    async fn set_status(
        pool: &PgPool,
        lesson_id: i32,
        status: LessonStatus,
    ) -> Result<Self, Error> {
        query_as!(
            Self,
            r#"
            UPDATE lesson
            SET status=$1
            WHERE id=$2
            RETURNING
                id,
                text,
                status as "status: _",
                created_at
            "#,
            status as LessonStatus,
            lesson_id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| Error::SetLessonStatus(e, lesson_id, status))
    }

    fn keyboard(
        &self,
        status_range: LessonStatusRange,
        lang: Lang,
        is_moderator: bool,
    ) -> InlineKeyboardMarkup {
        let mut line = vec![InlineKeyboardButton::callback(
            TEXT.next_lesson.to(lang),
            LessonReadOptions::new(status_range, Some(self.id)).to_command(),
        )];
        if is_moderator {
            for (status, label) in [
                (LessonStatus::Approved, "👍 Approve"),
                (LessonStatus::Rejected, "👎 Reject"),
                (LessonStatus::Best, "🏆 Mark best"),
            ] {
                if self.status != status {
                    line.push(InlineKeyboardButton::callback(
                        label,
                        SetLessonStatus::new(status_range, self.id, status).to_command(),
                    ));
                }
            }
        }
        line.push(InlineKeyboardButton::callback(TEXT.help.to(lang), "/help"));
        InlineKeyboardMarkup::new(vec![line])
    }

    fn message(&self, is_moderator: bool) -> String {
        if is_moderator {
            format!(
                "{}\n\nid: {}, status: {}, created: {} ago",
                self.text,
                self.id,
                self.status.as_ref(),
                timeago(self.created_at),
            )
        } else {
            self.text.to_owned()
        }
    }
}

impl LessonStatusRange {
    fn range(self) -> (LessonStatus, LessonStatus) {
        match self {
            Self::Rejected => (LessonStatus::Rejected, LessonStatus::Rejected),
            Self::New => (LessonStatus::New, LessonStatus::New),
            Self::Approved => (LessonStatus::Approved, LessonStatus::Best),
            Self::Best => (LessonStatus::Best, LessonStatus::Best),
            Self::All => (LessonStatus::New, LessonStatus::Best),
        }
    }
}

fn timeago(dt: OffsetDateTime) -> String {
    humantime::format_duration((OffsetDateTime::now_utc() - dt).unsigned_abs())
        .to_string()
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lesson_read_options_from_command() {
        assert_eq!(LessonReadOptions::from_command("/unknown"), None);

        assert_eq!(
            LessonReadOptions::from_command("/view").unwrap(),
            LessonReadOptions::default()
        );
        assert!(LessonReadOptions::from_command("/view unkonwn").is_none());
        assert_eq!(
            LessonReadOptions::from_command("/view best").unwrap(),
            LessonReadOptions::new(LessonStatusRange::Best, None)
        );

        assert!(LessonReadOptions::from_command("/view best bad").is_none());
        assert_eq!(
            LessonReadOptions::from_command("/view best 3").unwrap(),
            LessonReadOptions::new(LessonStatusRange::Best, Some(3))
        );
        assert_eq!(
            LessonReadOptions::from_command("/view approved 5").unwrap(),
            LessonReadOptions::new(LessonStatusRange::Approved, Some(5))
        );
        assert_eq!(
            LessonReadOptions::from_command("/view best 35").unwrap(),
            LessonReadOptions::new(LessonStatusRange::Best, Some(35))
        );
    }

    #[test]
    fn lesson_read_options_to_command() {
        assert_eq!(LessonReadOptions::default().to_command(), "/view approved");
        assert_eq!(
            LessonReadOptions::new(LessonStatusRange::Best, Some(35)).to_command(),
            "/view best 35"
        );
    }

    #[test]
    fn set_lesson_status_from_command() {
        assert!(SetLessonStatus::from_command("/unknown").is_none());
        assert!(SetLessonStatus::from_command("/set-lesson-status").is_none());
        assert!(SetLessonStatus::from_command("/set-lesson-status foo bar").is_none());
        assert!(SetLessonStatus::from_command("/set-lesson-status 1 bar").is_none());
        assert!(SetLessonStatus::from_command("/set-lesson-status foo best").is_none());
        assert_eq!(
            SetLessonStatus::from_command("/set-lesson-status approved 1 best").unwrap(),
            SetLessonStatus::new(LessonStatusRange::Approved, 1, LessonStatus::Best)
        );
    }
}
