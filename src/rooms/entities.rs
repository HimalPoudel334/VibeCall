use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::shared::response::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub room_type: RoomType,
    pub created_by: i32,
    pub description: Option<String>,
    pub max_participants: i32,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct RoomInfo {
    pub id: i32,
    pub user_count: usize,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoomMember {
    pub room_id: String,
    pub user_id: i32,
    pub joined_at: chrono::NaiveDateTime,
    pub role: RoomMemberRole,
    pub left_at: Option<chrono::NaiveDateTime>,
    pub is_muted: bool,
    pub is_video_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RoomType {
    Public,
    Private,
    OneOnOne,
    Group,
    Meeting,
    Instant,
}

impl FromStr for RoomType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "public" => Ok(RoomType::Public),
            "private" => Ok(RoomType::Private),
            "one_on_one" => Ok(RoomType::OneOnOne),
            "group" => Ok(RoomType::Group),
            "meeting" => Ok(RoomType::Meeting),
            "instant" => Ok(RoomType::Instant),
            _ => Err(AppError::Validation(format!("Invalid room type: '{}'", s))),
        }
    }
}

impl fmt::Display for RoomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RoomType::Public => "public",
            RoomType::Private => "private",
            RoomType::OneOnOne => "one_on_one",
            RoomType::Group => "group",
            RoomType::Meeting => "meeting",
            RoomType::Instant => "instant",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RoomMemberRole {
    Owner,
    Moderator,
    Participant,
}

impl fmt::Display for RoomMemberRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Owner => "owner",
            Self::Moderator => "moderator",
            Self::Participant => "participant",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for RoomMemberRole {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Self::Owner),
            "moderator" => Ok(Self::Moderator),
            "participant" => Ok(Self::Participant),
            _ => Err(AppError::Validation(format!("Invalid role: '{}'", s))),
        }
    }
}
