use std::sync::Arc;

use async_trait::async_trait;
use domain::crew::AstronautId;
use domain::wellbeing::WellbeingNote;

use super::Authorizer;
use crate::error::AppResult;
use crate::ports::inbound::WellbeingUseCase;
use crate::ports::outbound::{CrewRepository, WellbeingRepository};
use crate::security::Principal;

pub struct WellbeingService {
    authorizer: Authorizer,
    notes: Arc<dyn WellbeingRepository>,
}

impl WellbeingService {
    pub fn new(crew: Arc<dyn CrewRepository>, notes: Arc<dyn WellbeingRepository>) -> Self {
        Self {
            authorizer: Authorizer::new(crew),
            notes,
        }
    }
}

#[async_trait]
impl WellbeingUseCase for WellbeingService {
    async fn latest_note(
        &self,
        principal: &Principal,
        id: AstronautId,
    ) -> AppResult<Option<WellbeingNote>> {
        self.authorizer.astronaut_visible_to(principal, id).await?;
        Ok(self.notes.latest_note(id).await?)
    }
}
