use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    types::Uuid,
};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub password_salt: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
