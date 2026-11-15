use crate::body::BodyRegion;
use crate::metrics::{Bounds, ChartKind, ComponentDefinition, DemoSignal, MetricDefinition};

/// Muscle volume change vs. the pre-flight baseline (ultrasound / MRI), in %.
/// Postural muscles (calf, back, thigh) atrophy fastest without countermeasures.
pub fn definition() -> MetricDefinition {
    let muscle = |key, label, region, drift| {
        ComponentDefinition::new(
            key,
            label,
            "%",
            region,
            Bounds::between(-60.0, 25.0),
            Bounds::at_least(-5.0),
            Bounds::at_least(-10.0),
            1,
            DemoSignal {
                baseline: 0.0,
                spread: 0.5,
                drift_per_day: drift,
                noise: 0.5,
            },
        )
    };
    MetricDefinition::builder("muscle_atrophy", "Zanik mięśni")
        .description("Zmiana objętości mięśni względem pomiaru przed lotem.")
        .chart(ChartKind::Body)
        .group("physiology")
        .sampling_hours(24 * 7)
        .component(muscle(
            "deltoids",
            "Mięśnie naramienne",
            BodyRegion::Deltoids,
            -0.01,
        ))
        .component(muscle(
            "erector_spinae",
            "Prostownik grzbietu",
            BodyRegion::ErectorSpinae,
            -0.035,
        ))
        .component(muscle(
            "gluteus",
            "Mięśnie pośladkowe",
            BodyRegion::Gluteus,
            -0.03,
        ))
        .component(muscle(
            "quadriceps",
            "Mięsień czworogłowy uda",
            BodyRegion::Quadriceps,
            -0.03,
        ))
        .component(muscle(
            "hamstrings",
            "Mięśnie kulszowo-goleniowe",
            BodyRegion::Hamstrings,
            -0.035,
        ))
        .component(muscle(
            "triceps_surae",
            "Mięśnie łydki",
            BodyRegion::Calves,
            -0.045,
        ))
        .build()
}
