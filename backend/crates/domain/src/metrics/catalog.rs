use super::definition::{MetricCode, MetricDefinition};
use super::definitions;
use super::group::MetricGroup;
use crate::error::DomainError;

/// Registry of all known health metrics, in display order.
#[derive(Debug, Clone)]
pub struct MetricCatalog {
    groups: Vec<MetricGroup>,
    metrics: Vec<MetricDefinition>,
}

impl MetricCatalog {
    /// The built-in tabs and metrics. **To add a health parameter, create a
    /// file in `metrics/definitions/` and add one line here; to add a tab,
    /// add one `MetricGroup` line.**
    pub fn builtin() -> Self {
        Self::new(
            vec![
                MetricGroup::new(
                    "environment",
                    "Środowisko zewnętrzne",
                    "Warunki na pokładzie: powietrze, powierzchnie, promieniowanie, grawitacja.",
                ),
                // Default tab: the astronaut's own vital signs are the core of
                // the app. Urgent issues from other tabs are already surfaced
                // by the alert banner above the tabs, so opening on physiology
                // shows the most relevant data without overloading the view.
                MetricGroup::new(
                    "physiology",
                    "Parametry fizjologiczne",
                    "Parametry życiowe, badania laboratoryjne, mięśnie i kości.",
                )
                .default_tab(),
                MetricGroup::new("wellbeing", "Samopoczucie", "Stan psychiczny i jakość snu."),
            ],
            vec![
                definitions::air_microbes::definition(),
                definitions::surface_microbes::definition(),
                definitions::radiation::definition(),
                definitions::gravity::definition(),
                definitions::heart_rate::definition(),
                definitions::body_temperature::definition(),
                definitions::blood_pressure::definition(),
                definitions::muscle_atrophy::definition(),
                definitions::bone_density::definition(),
                definitions::blood_panel::definition(),
                definitions::urine_panel::definition(),
                definitions::sleep::definition(),
            ],
        )
    }

    pub fn new(groups: Vec<MetricGroup>, metrics: Vec<MetricDefinition>) -> Self {
        let mut seen = std::collections::HashSet::new();
        for m in &metrics {
            assert!(seen.insert(m.code.clone()), "duplicate metric `{}`", m.code);
            assert!(
                groups.iter().any(|g| g.code == m.group),
                "metric `{}` refers to unknown group `{}`",
                m.code,
                m.group
            );
        }
        assert_eq!(
            groups.iter().filter(|g| g.default).count(),
            1,
            "exactly one default group"
        );
        Self { groups, metrics }
    }

    pub fn groups(&self) -> &[MetricGroup] {
        &self.groups
    }

    pub fn all(&self) -> &[MetricDefinition] {
        &self.metrics
    }

    pub fn get(&self, code: &MetricCode) -> Option<&MetricDefinition> {
        self.metrics.iter().find(|m| &m.code == code)
    }

    pub fn get_str(&self, code: &str) -> Option<&MetricDefinition> {
        self.metrics.iter().find(|m| m.code.as_str() == code)
    }

    pub fn resolve(&self, code: &str) -> Result<&MetricDefinition, DomainError> {
        self.get_str(code)
            .ok_or_else(|| DomainError::UnknownMetric(code.to_owned()))
    }
}
