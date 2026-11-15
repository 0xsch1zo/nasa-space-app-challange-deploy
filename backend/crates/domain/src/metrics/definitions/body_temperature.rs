use crate::metrics::{Bounds, ChartKind, MetricDefinition};

use super::reading;

/// Core body temperature (wearable sensor). Long missions show a slow rise
/// of ~0.5–1 °C ("space fever"), so it is tracked daily.
pub fn definition() -> MetricDefinition {
    MetricDefinition::builder("body_temperature", "Temperatura ciała")
        .description("Temperatura głęboka ciała z czujnika noszonego, średnia poranna.")
        .chart(ChartKind::Line)
        .group("physiology")
        .sampling_hours(24)
        .component(reading(
            "core",
            "Temperatura",
            "°C",
            Bounds::between(30.0, 43.0),
            Bounds::between(36.0, 37.5),
            Bounds::between(35.5, 38.0),
            1,
            36.8,
            0.12,
            0.0025,
        ))
        .build()
}
