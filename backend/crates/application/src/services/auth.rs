use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{AppError, AppResult};
use crate::ports::inbound::{AuthUseCase, LoginResult};
use crate::ports::outbound::{PasswordHasher, TokenService, UserRepository};
use crate::security::Principal;

pub struct AuthService {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
}

impl AuthService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            users,
            hasher,
            tokens,
        }
    }
}

#[async_trait]
impl AuthUseCase for AuthService {
    async fn login(&self, username: &str, password: &str) -> AppResult<LoginResult> {
        let user = self
            .users
            .find_by_username(username.trim())
            .await?
            .ok_or(AppError::InvalidCredentials)?;
        if !self.hasher.verify(password, &user.password_hash) {
            return Err(AppError::InvalidCredentials);
        }
        let principal = Principal {
            user_id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
            station_id: user.station_id,
            astronaut_id: user.astronaut_id,
        };
        let token = self.tokens.issue(&principal)?;
        Ok(LoginResult { token, principal })
    }

    fn authenticate(&self, bearer_token: &str) -> AppResult<Principal> {
        self.tokens.verify(bearer_token)
    }
}
