use actix_session::Session;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::pages::PageId;

pub const SESSION_KEY: &str = "user_session";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSession {
    pub current_page: PageId,
}

impl UserSession {
    pub fn new(starting_page: &str) -> Self {
        UserSession {
            current_page: PageId::from(starting_page),
        }
    }
}

#[derive(Deserialize)]
pub struct UserAction {
    pub go_to: String, // direction of movement
}

/// Retrieve session or create a new one if missing
pub fn get_or_create_user_session(
    session: &Session,
    start_page: &str,
) -> Result<UserSession, AppError> {
    match session.get::<UserSession>(SESSION_KEY) {
        Ok(Some(val)) => Ok(val),
        Ok(None) => create_new_session(session, start_page),
        Err(e) => Err(AppError::SessionError(format!(
            "Failed to retrieve session: {e}",
        ))),
    }
}

/// Helper function for creating a new session
fn create_new_session(session: &Session, start_page: &str) -> Result<UserSession, AppError> {
    let new_session = UserSession::new(start_page);
    session
        .insert(SESSION_KEY, &new_session)
        .map_err(|e| AppError::SessionError(format!("Failed to insert new session: {e}")))
        .map(|_| new_session)
}

/// Save the session back to actix
pub fn set_user_session(session: &Session, user_session: &UserSession) {
    let _ = session.insert(SESSION_KEY, user_session);
}
