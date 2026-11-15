use crate::body::BodyRegion;
use crate::metrics::{Bounds, ChartKind, ComponentDefinition, DemoSignal, MetricDefinition};

/// Resting arterial blood pressure (cuff measurement, morning).
/// Microgravity shifts fluids headward; sustained elevation is monitored.
pub fn definition() -> MetricDefinition {
    MetricDefinition::builder("blood_pressure", "Ciśnienie krwi")
        .description("Spoczynkowe ciśnienie tętnicze mierzone rano mankietem.")
        .chart(ChartKind::Line)
        .sampling_hours(24)
        .group("physiology")
        .component(ComponentDefinition::new(
            "systolic",
            "Skurczowe",
            "mmHg",
            BodyRegion::Heart,
            Bounds::between(40.0, 260.0),
            Bounds::between(90.0, 129.0),
            Bounds::between(85.0, 139.0),
            0,
            DemoSignal {
                baseline: 116.0,
                spread: 5.0,
                drift_per_day: 0.03,
                noise: 4.5,
            },
        ))
        .component(ComponentDefinition::new(
            "diastolic",
            "Rozkurczowe",
            "mmHg",
            BodyRegion::Heart,
            Bounds::between(20.0, 160.0),
            Bounds::between(60.0, 84.0),
            Bounds::between(55.0, 89.0),
            0,
            DemoSignal {
                baseline: 75.0,
                spread: 3.5,
                drift_per_day: 0.02,
                noise: 3.5,
            },
        ))
        .build()
}
