use crate::metrics::{Bounds, ChartKind, MetricDefinition, MetricScope};

use super::reading;

/// Quasi-steady residual acceleration (microgravity level), daily mean.
/// Reboosts and dockings show up as short excursions.
pub fn definition() -> MetricDefinition {
    MetricDefinition::builder("gravity", "Natężenie pola grawitacyjnego")
        .description("Średnie dobowe przyspieszenie resztkowe na pokładzie (mikrograwitacja).")
        .chart(ChartKind::Line)
        .group("environment")
        .scope(MetricScope::Station)
        .sampling_hours(24)
        .component(reading(
            "residual",
            "Przyspieszenie resztkowe",
            "µg",
            Bounds::between(0.0, 1_000_000.0),
            Bounds::at_most(10.0),
            Bounds::at_most(100.0),
            1,
            1.2,
            0.25,
            0.0,
        ))
        .build()
}
