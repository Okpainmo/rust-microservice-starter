use crate::core::structs::session::Session;
use crate::core::structs::sub_session::SubSession;
use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SessionResponseCore {
    id: Uuid,
    creation_order: i64,
    user_id: i64,
    expires_at: NaiveDateTime,
    status: String,
    revoked_at: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    sub_sessions: Vec<SubSession>,
}

impl SessionResponseCore {
    pub fn new(session: Session, sub_sessions: Vec<SubSession>) -> Self {
        Self {
            id: session.id,
            creation_order: session.creation_order,
            user_id: session.user_id,
            expires_at: session.expires_at,
            status: session.status,
            revoked_at: session.revoked_at,
            created_at: session.created_at,
            updated_at: session.updated_at,
            sub_sessions,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionWithSubSessions {
    session: SessionResponseCore,
}

impl SessionWithSubSessions {
    pub fn new(session: Session, sub_sessions: Vec<SubSession>) -> Self {
        Self {
            session: SessionResponseCore::new(session, sub_sessions),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionsResponseCore {
    data: Vec<SessionWithSubSessions>,
    session_id: String,
    access_token: String,
    refresh_token: String,
}

impl SessionsResponseCore {
    pub fn new(
        data: Vec<SessionWithSubSessions>,
        session_id: String,
        access_token: String,
        refresh_token: String,
    ) -> Self {
        Self {
            data,
            session_id,
            access_token,
            refresh_token,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionDataResponseCore {
    data: SessionWithSubSessions,
    session_id: String,
    access_token: String,
    refresh_token: String,
}

impl SessionDataResponseCore {
    pub fn new(
        data: SessionWithSubSessions,
        session_id: String,
        access_token: String,
        refresh_token: String,
    ) -> Self {
        Self {
            data,
            session_id,
            access_token,
            refresh_token,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionsResponse {
    response_message: String,
    response: Option<SessionsResponseCore>,
    error: Option<String>,
}

impl SessionsResponse {
    pub fn success(response_message: impl Into<String>, response: SessionsResponseCore) -> Self {
        Self {
            response_message: response_message.into(),
            response: Some(response),
            error: None,
        }
    }

    pub fn failure(response_message: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            response_message: response_message.into(),
            response: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    response_message: String,
    response: Option<SessionDataResponseCore>,
    error: Option<String>,
}

impl SessionResponse {
    pub fn success(response_message: impl Into<String>, response: SessionDataResponseCore) -> Self {
        Self {
            response_message: response_message.into(),
            response: Some(response),
            error: None,
        }
    }

    pub fn failure(response_message: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            response_message: response_message.into(),
            response: None,
            error: Some(error.into()),
        }
    }
}
