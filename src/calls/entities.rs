use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{calls::contract::PublicUser, shared::response::AppError, users::User};

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
            "active" => Ok(CallStatus::Missed),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    // User wants to join the call
    #[serde(rename = "join")]
    Join { room_id: String, user_id: i32 },

    // WebRTC offer (initiating connection)
    #[serde(rename = "offer")]
    Offer {
        to_user_id: i32,
        sdp: String, // Session Description Protocol
    },

    // WebRTC answer (accepting connection)
    #[serde(rename = "answer")]
    Answer { to_user_id: i32, sdp: String },

    // ICE candidates (network routing info)
    #[serde(rename = "ice-candidate")]
    IceCandidate { to_user_id: i32, candidate: String },

    // User leaving call
    #[serde(rename = "leave")]
    Leave { room_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "user-joined")]
    UserJoined { user: PublicUser, users: Vec<User> },

    #[serde(rename = "user-left")]
    UserLeft {
        user_id: i32,
        users: Vec<PublicUser>,
    }, 

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
