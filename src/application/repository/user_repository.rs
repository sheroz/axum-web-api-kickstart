use chrono::Utc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{domain::model::user::User, shared::state::SharedState};

pub async fn all_users(state: &SharedState) -> Option<Vec<User>> {
    match query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&state.pgpool)
        .await
    {
        Ok(users) => Some(users),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

pub async fn add_user(user: User, state: &SharedState) -> Option<User> {
    let time_now = Utc::now().naive_utc();
    tracing::trace!("user: {:#?}", user);
    let query_add = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (id,
         username,
         email,
         password_hash,
         password_salt,
         created_at,
         updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         RETURNING users.*"#,
    )
    .bind(user.id)
    .bind(user.username)
    .bind(user.email)
    .bind(user.password_hash)
    .bind(user.password_salt)
    .bind(time_now)
    .bind(time_now)
    .fetch_one(&state.pgpool)
    .await;

    match query_add {
        Ok(user) => Some(user),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

pub async fn get_user(id: Uuid, state: &SharedState) -> Option<User> {
    let query_get = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pgpool)
        .await;

    match query_get {
        Ok(user) => Some(user),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

pub async fn get_user_by_username(username: &str, state: &SharedState) -> Option<User> {
    let query_get = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(&state.pgpool)
        .await;

    match query_get {
        Ok(user) => Some(user),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

pub async fn update_user(id: Uuid, user: User, state: &SharedState) -> Option<User> {
    tracing::trace!("user: {:#?}", user);
    let time_now = Utc::now().naive_utc();
    let query_update = sqlx::query_as::<_, User>(
        r#"UPDATE users
         SET id = $1,
         username = $2,
         email = $3,
         password_hash = $4,
         password_salt = $5,
         updated_at = $6
         WHERE id = $7
         RETURNING users.*"#,
    )
    .bind(user.id)
    .bind(user.username)
    .bind(user.email)
    .bind(user.password_hash)
    .bind(user.password_salt)
    .bind(time_now)
    .bind(id)
    .fetch_one(&state.pgpool)
    .await;

    match query_update {
        Ok(user) => Some(user),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

pub async fn delete_user(id: Uuid, state: &SharedState) -> Option<bool> {
    let query_delete = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.pgpool)
        .await;

    match query_delete {
        Ok(row) => Some(row.rows_affected() == 1),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}
