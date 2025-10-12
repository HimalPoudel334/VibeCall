use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::shared::response::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum CallStatus {
    Initiated,
    Ringing,
    Active,
    Ended,
    Missed,
    Rejected,
    Failed,
}

impl FromStr for CallStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "initiated" => Ok(CallStatus::Initiated),
            "ringing" => Ok(CallStatus::Ringing),
            "active" => Ok(CallStatus::Active),
            "ended" => Ok(CallStatus::Ended),
            "missed" => Ok(CallStatus::Missed),
            "rejected" => Ok(CallStatus::Rejected),
            "failed" => Ok(CallStatus::Failed),
            _ => Err(AppError::Validation(
                "Invalid call status provided! Valid values are: 'initiated', 'ringing', 'active', 'ended', 'missed', 'rejected', 'failed'".to_string(),
            )),
        }
    }
}

impl fmt::Display for CallStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CallStatus::Initiated => "initiated",
            CallStatus::Ringing => "ringing",
            CallStatus::Active => "active",
            CallStatus::Ended => "ended",
            CallStatus::Missed => "missed",
            CallStatus::Rejected => "rejected",
            CallStatus::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Call {
    pub id: i32,
    pub room_id: String,
    pub caller_id: i32,
    pub status: CallStatus,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CallParticipant {
    pub call_id: i32,
    pub user_id: i32,
    pub joined_at: String,
    pub left_at: Option<String>,
    pub duration: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    #[serde(rename = "join")]
    Join { room_id: String, user_id: i32 },

    #[serde(rename = "leave")]
    Leave { room_id: String },

    #[serde(rename = "offer")]
    Offer { target_user_id: i32, sdp: String },

    #[serde(rename = "answer")]
    Answer { target_user_id: i32, sdp: String },

    #[serde(rename = "ice_candidate")]
    IceCandidate {
        target_user_id: i32,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "user-joined")]
    UserJoined {
        user_id: i32,
        users: Vec<(i32, String)>,
    },

    #[serde(rename = "user-left")]
    UserLeft { user_id: i32, user_name: String },

    #[serde(rename = "offer")]
    Offer { from: i32, sdp: String },

    #[serde(rename = "answer")]
    Answer { from: i32, sdp: String },

    #[serde(rename = "ice-candidate")]
    IceCandidate {
        from: i32,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },

    #[serde(rename = "error")]
    Error { message: String },
}
