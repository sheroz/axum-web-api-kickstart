use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    types::Uuid,
};

#[allow(non_snake_case)]
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub pswd_hash: String,
    pub pswd_salt: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
