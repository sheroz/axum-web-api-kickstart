use chrono::Utc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    application::{app_const::USER_ROLE_GUEST, repository::RepositoryResult, state::SharedState},
    domain::models::user::User,
};

pub async fn get_all(state: &SharedState) -> RepositoryResult<Vec<User>> {
    let users = query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&state.pgpool)
        .await?;

    Ok(users)
}

pub async fn add(user: User, state: &SharedState) -> RepositoryResult<User> {
    let time_now = Utc::now().naive_utc();
    tracing::trace!("user: {:#?}", user);
    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (id,
         username,
         email,
         password_hash,
         password_salt,
         active,
         roles,
         created_at,
         updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         RETURNING users.*"#,
    )
    .bind(user.id)
    .bind(user.username)
    .bind(user.email)
    .bind(user.password_hash)
    .bind(user.password_salt)
    .bind(true)
    .bind(USER_ROLE_GUEST)
    .bind(time_now)
    .bind(time_now)
    .fetch_one(&state.pgpool)
    .await?;

    Ok(user)
}

pub async fn get_by_id(id: Uuid, state: &SharedState) -> RepositoryResult<User> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pgpool)
        .await?;
    Ok(user)
}

pub async fn get_user_by_username(username: &str, state: &SharedState) -> RepositoryResult<User> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(&state.pgpool)
        .await?;

    Ok(user)
}

pub async fn update(id: Uuid, user: User, state: &SharedState) -> RepositoryResult<User> {
    tracing::trace!("user: {:#?}", user);
    let time_now = Utc::now().naive_utc();
    let user = sqlx::query_as::<_, User>(
        r#"UPDATE users
         SET id = $1,
         username = $2,
         email = $3,
         password_hash = $4,
         password_salt = $5,
         active = $6,
         roles = $7,
         updated_at = $8
         WHERE id = $9
         RETURNING users.*"#,
    )
    .bind(user.id)
    .bind(user.username)
    .bind(user.email)
    .bind(user.password_hash)
    .bind(user.password_salt)
    .bind(user.active)
    .bind(user.roles)
    .bind(time_now)
    .bind(id)
    .fetch_one(&state.pgpool)
    .await?;

    Ok(user)
}

pub async fn delete(id: Uuid, state: &SharedState) -> RepositoryResult<bool> {
    let query_result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.pgpool)
        .await?;

    Ok(query_result.rows_affected() == 1)
}
