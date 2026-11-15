//! Wellbeing notes: short, gently worded descriptions of an astronaut's
//! mental state. They are produced outside this system (from collected data)
//! and only stored and displayed here. Deliberately never a source of alerts.

use chrono::{DateTime, Utc};

use crate::crew::AstronautId;

#[derive(Debug, Clone, PartialEq)]
pub struct WellbeingNote {
    pub astronaut_id: AstronautId,
    pub written_at: DateTime<Utc>,
    pub body: String,
}
