use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    #[error("unknown health metric `{0}`")]
    UnknownMetric(String),
    #[error("invalid metric code `{0}` (allowed: lowercase letters, digits, underscore)")]
    InvalidMetricCode(String),
    #[error("metric `{metric}`: missing component `{component}`")]
    MissingComponent { metric: String, component: String },
    #[error("metric `{metric}`: component `{component}` is computed and must not be sent")]
    DerivedComponentProvided { metric: String, component: String },
    #[error("metric `{0}` is recorded for a different subject (astronaut vs station)")]
    ScopeMismatch(String),
    #[error("metric `{metric}`: unknown component `{component}`")]
    UnknownComponent { metric: String, component: String },
    #[error("metric `{metric}`: value {value} of `{component}` is outside the plausible range [{min}, {max}]")]
    ImplausibleValue {
        metric: String,
        component: String,
        value: f64,
        min: f64,
        max: f64,
    },
}
