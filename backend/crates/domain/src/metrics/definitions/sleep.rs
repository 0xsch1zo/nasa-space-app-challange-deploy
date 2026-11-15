use crate::metrics::{AlertPolicy, Bounds, ChartKind, MetricDefinition, ScoreRule};

use super::reading;

/// Nightly sleep (actigraphy + sleep tracker). The overall score is the sum
/// of the per-parameter scores (max 100) and is computed by the domain.
/// Alerts are raised on the overall score only: single sub-parameters are
/// informative, not alarming.
pub fn definition() -> MetricDefinition {
    let b = Bounds::between;
    let score = |max_points, zero_at, full_at| ScoreRule {
        max_points,
        zero_at,
        full_at,
    };
    MetricDefinition::builder("sleep", "Sen")
        .description("Ogólna punktacja snu oraz parametry ostatniej nocy.")
        .chart(ChartKind::Line)
        .group("wellbeing")
        .sampling_hours(24)
        .alert_policy(AlertPolicy::Only(vec!["score".into()]))
        .component(
            reading(
                "total_sleep",
                "Czas snu",
                "h",
                b(0.0, 16.0),
                b(6.0, 9.5),
                b(5.0, 10.5),
                1,
                6.9,
                0.45,
                -0.001,
            )
            .scored(score(30.0, 4.0, 7.5)),
        )
        .component(
            reading(
                "efficiency",
                "Wydajność snu",
                "%",
                b(30.0, 100.0),
                Bounds::at_least(85.0),
                Bounds::at_least(75.0),
                0,
                89.0,
                2.5,
                0.0,
            )
            .scored(score(20.0, 65.0, 92.0)),
        )
        .component(
            reading(
                "deep_sleep",
                "Sen głęboki",
                "%",
                b(0.0, 50.0),
                b(13.0, 25.0),
                b(10.0, 30.0),
                0,
                17.0,
                2.0,
                0.0,
            )
            .scored(score(15.0, 5.0, 18.0)),
        )
        .component(
            reading(
                "rem_sleep",
                "Faza REM",
                "%",
                b(0.0, 50.0),
                b(18.0, 28.0),
                b(14.0, 32.0),
                0,
                21.0,
                2.0,
                0.0,
            )
            .scored(score(15.0, 8.0, 22.0)),
        )
        .component(
            reading(
                "latency",
                "Czas zasypiania",
                "min",
                b(0.0, 180.0),
                Bounds::at_most(20.0),
                Bounds::at_most(40.0),
                0,
                13.0,
                4.0,
                0.0,
            )
            .scored(score(10.0, 60.0, 10.0)),
        )
        .component(
            reading(
                "awakenings",
                "Wybudzenia",
                "",
                b(0.0, 30.0),
                Bounds::at_most(3.0),
                Bounds::at_most(6.0),
                0,
                1.6,
                0.8,
                0.0,
            )
            .scored(score(10.0, 8.0, 1.0)),
        )
        .derived_score(
            "score",
            "Punktacja snu",
            Bounds::at_least(70.0),
            Bounds::at_least(50.0),
        )
        .build()
}
