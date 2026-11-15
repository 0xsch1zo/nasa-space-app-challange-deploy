//! Authenticated identity and the single source of truth for data access rules.

use domain::crew::{Astronaut, AstronautId, Role, StationId, UserId};

/// The authenticated caller, reconstructed from a verified JWT.
#[derive(Debug, Clone, PartialEq)]
pub struct Principal {
    pub user_id: UserId,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub station_id: StationId,
    pub astronaut_id: Option<AstronautId>,
}

/// Access rules for health data:
/// * an astronaut sees only their own profile,
/// * a supervisor sees every astronaut assigned to their station.
pub struct AccessPolicy;

impl AccessPolicy {
    pub fn can_view(principal: &Principal, astronaut: &Astronaut) -> bool {
        match principal.role {
            Role::Astronaut => principal.astronaut_id == Some(astronaut.id),
            Role::Supervisor => principal.station_id == astronaut.station_id,
        }
    }

    pub fn can_list_crew(principal: &Principal) -> bool {
        principal.role == Role::Supervisor
    }
}
