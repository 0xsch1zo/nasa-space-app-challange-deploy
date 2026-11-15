//! Crew, stations and user accounts.

use chrono::NaiveDate;
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(AstronautId);
id_type!(StationId);
id_type!(UserId);

#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    pub id: StationId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Astronaut {
    pub id: AstronautId,
    pub station_id: StationId,
    pub full_name: String,
    pub agency: String,
    pub specialty: String,
    pub mission_start: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Sees only their own health profile.
    Astronaut,
    /// Flight surgeon / medical officer: sees every astronaut on their station.
    Supervisor,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Astronaut => "astronaut",
            Role::Supervisor => "supervisor",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "astronaut" => Some(Role::Astronaut),
            "supervisor" => Some(Role::Supervisor),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub role: Role,
    pub station_id: StationId,
    /// Set for astronaut accounts, `None` for supervisors.
    pub astronaut_id: Option<AstronautId>,
}

/// Owner of a metric history: an astronaut or a whole station.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subject {
    Astronaut(AstronautId),
    Station(StationId),
}
