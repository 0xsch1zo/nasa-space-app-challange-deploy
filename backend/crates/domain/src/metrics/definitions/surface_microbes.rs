use crate::metrics::{Bounds, ChartKind, MetricDefinition, MetricScope};

use super::reading;

/// Weekly swab sampling of station surfaces, analysed for microbial load.
/// Each sampling location is a component; ISS limit is 10 000 CFU/100 cm².
pub fn definition() -> MetricDefinition {
    let site = |key, label, baseline| {
        reading(
            key,
            label,
            "CFU/100 cm²",
            Bounds::between(0.0, 1_000_000.0),
            Bounds::at_most(5000.0),
            Bounds::at_most(10_000.0),
            0,
            baseline,
            350.0,
            0.0,
        )
    };
    MetricDefinition::builder("surface_microbes", "Próbki z powierzchni stacji")
        .description(
            "Cotygodniowe wymazy z powierzchni stacji analizowane pod kątem liczby mikrobów.",
        )
        .chart(ChartKind::Bar)
        .group("environment")
        .scope(MetricScope::Station)
        .sampling_hours(24 * 7)
        .component(site("galley", "Kuchnia (stół)", 1500.0))
        .component(site("hygiene", "Węzeł sanitarny", 3100.0))
        .component(site("sleep_quarters", "Kabiny sypialne", 900.0))
        .component(site("exercise", "Strefa ćwiczeń", 2300.0))
        .build()
}
