use crate::body::BodyRegion;
use crate::metrics::{Bounds, ChartKind, ComponentDefinition, DemoSignal, MetricDefinition};

/// Bone mineral density expressed as a T-score per skeletal site.
/// In microgravity weight-bearing bones lose ~1–1.5 % BMD per month, while
/// non-weight-bearing sites (radius) barely change.
pub fn definition() -> MetricDefinition {
    let site = |key, label, region, baseline, drift| {
        ComponentDefinition::new(
            key,
            label,
            "T-score",
            region,
            Bounds::between(-6.0, 5.0),
            Bounds::at_least(-1.0),
            Bounds::at_least(-2.5),
            2,
            DemoSignal {
                baseline,
                spread: 0.35,
                drift_per_day: drift,
                noise: 0.03,
            },
        )
    };
    MetricDefinition::builder("bone_density", "Zanik kości")
        .description(
            "Gęstość mineralna kości (T-score) w miejscach najbardziej narażonych na utratę masy.",
        )
        .chart(ChartKind::Body)
        .group("physiology")
        .sampling_hours(24 * 14)
        .component(site(
            "lumbar_spine",
            "Kręgosłup lędźwiowy",
            BodyRegion::LumbarSpine,
            0.4,
            -0.0045,
        ))
        .component(site("pelvis", "Miednica", BodyRegion::Pelvis, 0.3, -0.0055))
        .component(site(
            "femoral_neck",
            "Szyjka kości udowej",
            BodyRegion::FemoralNeck,
            0.2,
            -0.006,
        ))
        .component(site(
            "distal_tibia",
            "Dalsza nasada piszczeli",
            BodyRegion::DistalTibia,
            0.3,
            -0.005,
        ))
        .component(site(
            "distal_radius",
            "Dalsza nasada kości promieniowej",
            BodyRegion::DistalRadius,
            0.3,
            -0.0005,
        ))
        .build()
}
