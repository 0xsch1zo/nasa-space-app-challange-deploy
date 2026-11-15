//! One file per health metric. Each exposes `pub fn definition() -> MetricDefinition`.

pub mod air_microbes;
pub mod blood_panel;
pub mod blood_pressure;
pub mod body_temperature;
pub mod bone_density;
pub mod gravity;
pub mod heart_rate;
pub mod muscle_atrophy;
pub mod radiation;
pub mod sleep;
pub mod surface_microbes;
pub mod urine_panel;

use crate::metrics::{Bounds, ComponentDefinition, DemoSignal};

/// Shorthand for a lab analyte or environment reading: no body region,
/// steady demo signal around `baseline`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn reading(
    key: &str,
    label: &str,
    unit: &str,
    plausible: Bounds,
    normal: Bounds,
    warning: Bounds,
    decimals: u8,
    baseline: f64,
    noise: f64,
    drift_per_day: f64,
) -> ComponentDefinition {
    ComponentDefinition::new(
        key,
        label,
        unit,
        None,
        plausible,
        normal,
        warning,
        decimals,
        DemoSignal {
            baseline,
            spread: noise * 0.6,
            drift_per_day,
            noise,
        },
    )
}
