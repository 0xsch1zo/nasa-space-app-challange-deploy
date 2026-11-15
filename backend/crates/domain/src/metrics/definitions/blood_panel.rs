use crate::metrics::{Bounds, ChartKind, MetricDefinition};

use super::reading;

/// Monthly blood test (in-flight centrifuge + analyser). Each analyte is a
/// component, so every exam is one reading and history = previous exams.
pub fn definition() -> MetricDefinition {
    let b = Bounds::between;
    MetricDefinition::builder("blood_panel", "Badanie krwi")
        .description("Morfologia i biochemia krwi z ostatniego pobrania.")
        .chart(ChartKind::Table)
        .group("physiology")
        .sampling_hours(24 * 30)
        .component(reading(
            "hemoglobin",
            "Hemoglobina",
            "g/dL",
            b(3.0, 25.0),
            b(12.0, 17.0),
            b(11.0, 18.5),
            1,
            14.3,
            0.35,
            -0.003,
        ))
        .component(reading(
            "hematocrit",
            "Hematokryt",
            "%",
            b(10.0, 70.0),
            b(36.0, 50.0),
            b(33.0, 53.0),
            1,
            42.5,
            1.0,
            -0.008,
        ))
        .component(reading(
            "wbc",
            "Leukocyty (WBC)",
            "10⁹/L",
            b(0.5, 50.0),
            b(4.0, 10.0),
            b(3.0, 12.0),
            1,
            6.4,
            0.5,
            0.0,
        ))
        .component(reading(
            "platelets",
            "Płytki krwi",
            "10⁹/L",
            b(10.0, 1000.0),
            b(150.0, 400.0),
            b(120.0, 450.0),
            0,
            245.0,
            15.0,
            0.0,
        ))
        .component(reading(
            "glucose",
            "Glukoza (na czczo)",
            "mg/dL",
            b(20.0, 600.0),
            b(70.0, 99.0),
            b(60.0, 125.0),
            0,
            86.0,
            3.5,
            0.0,
        ))
        .component(reading(
            "creatinine",
            "Kreatynina",
            "mg/dL",
            b(0.1, 15.0),
            b(0.6, 1.2),
            b(0.5, 1.5),
            2,
            0.88,
            0.04,
            0.0,
        ))
        .component(reading(
            "sodium",
            "Sód",
            "mmol/L",
            b(100.0, 180.0),
            b(135.0, 145.0),
            b(132.0, 148.0),
            0,
            140.0,
            1.0,
            0.0,
        ))
        .component(reading(
            "potassium",
            "Potas",
            "mmol/L",
            b(1.5, 9.0),
            b(3.5, 5.0),
            b(3.2, 5.5),
            1,
            4.2,
            0.12,
            0.0,
        ))
        .component(reading(
            "calcium",
            "Wapń całkowity",
            "mg/dL",
            b(4.0, 16.0),
            b(8.6, 10.2),
            b(8.2, 10.8),
            1,
            9.4,
            0.12,
            0.001,
        ))
        .component(reading(
            "vitamin_d",
            "Witamina D (25-OH)",
            "ng/mL",
            b(2.0, 200.0),
            b(30.0, 100.0),
            b(20.0, 150.0),
            0,
            44.0,
            2.0,
            -0.03,
        ))
        .component(reading(
            "crp",
            "Białko C-reaktywne (CRP)",
            "mg/L",
            b(0.0, 300.0),
            Bounds::at_most(5.0),
            Bounds::at_most(10.0),
            1,
            1.2,
            0.3,
            0.0,
        ))
        .build()
}
