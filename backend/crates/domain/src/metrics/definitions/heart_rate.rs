use crate::body::BodyRegion;
use crate::metrics::{Bounds, ChartKind, ComponentDefinition, DemoSignal, MetricDefinition};

/// Resting heart rate, measured by the wearable each morning.
pub fn definition() -> MetricDefinition {
    MetricDefinition::builder("heart_rate", "Tętno spoczynkowe")
        .description("Spoczynkowa częstość akcji serca mierzona rano przez opaskę.")
        .chart(ChartKind::Line)
        .sampling_hours(24)
        .group("physiology")
        .component(ComponentDefinition::new(
            "resting",
            "Tętno",
            "bpm",
            BodyRegion::Heart,
            Bounds::between(25.0, 220.0),
            Bounds::between(50.0, 90.0),
            Bounds::between(45.0, 100.0),
            0,
            DemoSignal {
                baseline: 62.0,
                spread: 5.0,
                drift_per_day: 0.02,
                noise: 3.0,
            },
        ))
        .build()
}
