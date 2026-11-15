use std::sync::Arc;

use domain::crew::{Astronaut, AstronautId, Role};

use crate::error::{AppError, AppResult};
use crate::ports::outbound::CrewRepository;
use crate::security::{AccessPolicy, Principal};

/// Loads an astronaut and checks the [`AccessPolicy`]. Every use case that
/// touches a health profile goes through this, regardless of the adapter.
#[derive(Clone)]
pub struct Authorizer {
    crew: Arc<dyn CrewRepository>,
}

impl Authorizer {
    pub fn new(crew: Arc<dyn CrewRepository>) -> Self {
        Self { crew }
    }

    pub async fn astronaut_visible_to(
        &self,
        principal: &Principal,
        id: AstronautId,
    ) -> AppResult<Astronaut> {
        // Cheap early exit: astronauts may only ever ask about themselves.
        if principal.role == Role::Astronaut && principal.astronaut_id != Some(id) {
            return Err(AppError::Forbidden);
        }
        let astronaut = match self.crew.find_astronaut(id).await? {
            Some(a) => a,
            // An astronaut's own profile is gone: the session is stale.
            None if principal.astronaut_id == Some(id) => return Err(AppError::Unauthenticated),
            None => return Err(AppError::NotFound("astronaut")),
        };
        if !AccessPolicy::can_view(principal, &astronaut) {
            return Err(AppError::Forbidden);
        }
        Ok(astronaut)
    }
}
