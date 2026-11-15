//! Mapping of health data onto the human body (for the body visualisations).

use std::collections::BTreeMap;

use crate::metrics::{HealthStatus, MetricCatalog, MetricCode, Reading};

/// Anatomical parts the UI can draw. Each metric component may declare the
/// part it belongs to; body figures (muscles, bones, ...) are computed from that.
/// Adding a part = a variant here + a shape in the frontend `BodyFigure`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BodyRegion {
    Head,
    Heart,
    Chest,
    Abdomen,
    Arms,
    // muscles
    Deltoids,
    ErectorSpinae,
    Gluteus,
    Quadriceps,
    Hamstrings,
    Calves,
    // bones
    LumbarSpine,
    Pelvis,
    FemoralNeck,
    DistalTibia,
    DistalRadius,
}

impl BodyRegion {
    pub const ALL: [BodyRegion; 16] = [
        BodyRegion::Head,
        BodyRegion::Heart,
        BodyRegion::Chest,
        BodyRegion::Abdomen,
        BodyRegion::Arms,
        BodyRegion::Deltoids,
        BodyRegion::ErectorSpinae,
        BodyRegion::Gluteus,
        BodyRegion::Quadriceps,
        BodyRegion::Hamstrings,
        BodyRegion::Calves,
        BodyRegion::LumbarSpine,
        BodyRegion::Pelvis,
        BodyRegion::FemoralNeck,
        BodyRegion::DistalTibia,
        BodyRegion::DistalRadius,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            BodyRegion::Head => "head",
            BodyRegion::Heart => "heart",
            BodyRegion::Chest => "chest",
            BodyRegion::Abdomen => "abdomen",
            BodyRegion::Arms => "arms",
            BodyRegion::Deltoids => "deltoids",
            BodyRegion::ErectorSpinae => "erector_spinae",
            BodyRegion::Gluteus => "gluteus",
            BodyRegion::Quadriceps => "quadriceps",
            BodyRegion::Hamstrings => "hamstrings",
            BodyRegion::Calves => "calves",
            BodyRegion::LumbarSpine => "lumbar_spine",
            BodyRegion::Pelvis => "pelvis",
            BodyRegion::FemoralNeck => "femoral_neck",
            BodyRegion::DistalTibia => "distal_tibia",
            BodyRegion::DistalRadius => "distal_radius",
        }
    }
}

/// One value shown on a body region.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyFinding {
    pub metric: MetricCode,
    pub component: String,
    pub value: f64,
    pub status: HealthStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionSummary {
    pub region: BodyRegion,
    pub status: HealthStatus,
    pub findings: Vec<BodyFinding>,
}

/// Projects the latest reading of every metric onto body regions.
/// Regions without data are reported as `Unknown`.
pub fn body_map(catalog: &MetricCatalog, latest: &[(MetricCode, Reading)]) -> Vec<RegionSummary> {
    let mut regions: BTreeMap<BodyRegion, Vec<BodyFinding>> = BTreeMap::new();
    for (code, reading) in latest {
        let Some(def) = catalog.get(code) else {
            continue;
        };
        for assessed in def.assess(reading).components {
            let Some(region) = def.component(&assessed.key).and_then(|c| c.body_region) else {
                continue;
            };
            regions.entry(region).or_default().push(BodyFinding {
                metric: code.clone(),
                component: assessed.key,
                value: assessed.value,
                status: assessed.status,
            });
        }
    }
    BodyRegion::ALL
        .iter()
        .map(|&region| {
            let findings = regions.remove(&region).unwrap_or_default();
            let status = findings
                .iter()
                .map(|f| f.status)
                .max()
                .unwrap_or(HealthStatus::Unknown);
            RegionSummary {
                region,
                status,
                findings,
            }
        })
        .collect()
}
