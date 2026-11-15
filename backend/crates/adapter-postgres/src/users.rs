use application::ports::outbound::{InfraResult, UserRepository};
use async_trait::async_trait;
use domain::crew::{AstronautId, Role, StationId, User, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::infra;

#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

type UserRow = (Uuid, String, String, String, String, Uuid, Option<Uuid>);

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_username(&self, username: &str) -> InfraResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, username, display_name, password_hash, role, station_id, astronaut_id
             FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?;

        Ok(row.and_then(
            |(id, username, display_name, password_hash, role, station, astro)| {
                Some(User {
                    id: UserId(id),
                    username,
                    display_name,
                    password_hash,
                    role: Role::parse(&role)?,
                    station_id: StationId(station),
                    astronaut_id: astro.map(AstronautId),
                })
            },
        ))
    }
}
