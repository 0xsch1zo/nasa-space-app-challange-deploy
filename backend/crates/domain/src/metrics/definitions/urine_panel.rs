use crate::metrics::{Bounds, ChartKind, MetricDefinition};

use super::reading;

/// Urinalysis every two weeks. Urinary calcium and NTX track bone resorption
/// and kidney-stone risk, both raised in microgravity.
pub fn definition() -> MetricDefinition {
    let b = Bounds::between;
    MetricDefinition::builder("urine_panel", "Badanie moczu")
        .description("Analiza moczu (dobowa zbiórka) z markerami resorpcji kości.")
        .chart(ChartKind::Table)
        .group("physiology")
        .sampling_hours(24 * 15)
        .component(reading(
            "calcium",
            "Wapń (dobowy)",
            "mg/24h",
            b(0.0, 1500.0),
            b(100.0, 250.0),
            b(60.0, 300.0),
            0,
            185.0,
            10.0,
            0.15,
        ))
        .component(reading(
            "ntx",
            "NTX (resorpcja kości)",
            "nmol BCE/mmol Cr",
            b(0.0, 300.0),
            b(5.0, 65.0),
            b(3.0, 90.0),
            0,
            36.0,
            3.0,
            0.08,
        ))
        .component(reading(
            "specific_gravity",
            "Ciężar właściwy",
            "g/mL",
            b(1.0, 1.05),
            b(1.005, 1.03),
            b(1.002, 1.035),
            3,
            1.018,
            0.002,
            0.0,
        ))
        .component(reading(
            "ph",
            "pH",
            "pH",
            b(4.0, 9.0),
            b(5.0, 7.5),
            b(4.5, 8.0),
            1,
            6.1,
            0.25,
            0.0,
        ))
        .component(reading(
            "protein",
            "Białko",
            "mg/dL",
            b(0.0, 1000.0),
            Bounds::at_most(14.0),
            Bounds::at_most(30.0),
            0,
            6.0,
            1.5,
            0.0,
        ))
        .component(reading(
            "glucose",
            "Glukoza",
            "mg/dL",
            b(0.0, 1000.0),
            Bounds::at_most(15.0),
            Bounds::at_most(30.0),
            0,
            4.0,
            1.5,
            0.0,
        ))
        .build()
}
