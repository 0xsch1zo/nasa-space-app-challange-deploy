use crate::metrics::{Bounds, ChartKind, MetricDefinition, MetricScope};

use super::reading;

/// Absorbed dose rate on board (dosimeter network), daily total.
/// Typical LEO ≈ 0.3–0.6 mSv/day; solar particle events can multiply it.
pub fn definition() -> MetricDefinition {
    MetricDefinition::builder("radiation", "Promieniowanie na stacji")
        .description("Dobowa dawka promieniowania jonizującego na pokładzie (siwerty).")
        .chart(ChartKind::Area)
        .group("environment")
        .scope(MetricScope::Station)
        .sampling_hours(24)
        .component(reading(
            "dose_rate",
            "Dawka dobowa",
            "mSv/doba",
            Bounds::between(0.0, 1000.0),
            Bounds::at_most(1.0),
            Bounds::at_most(2.0),
            2,
            0.45,
            0.035,
            0.0,
        ))
        .build()
}
