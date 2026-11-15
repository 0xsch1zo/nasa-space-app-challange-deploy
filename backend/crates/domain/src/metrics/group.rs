/// A dashboard tab grouping related metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricGroup {
    pub code: String,
    pub name: String,
    pub description: String,
    /// Opened first when a health profile is shown. Exactly one per catalog.
    pub default: bool,
}

impl MetricGroup {
    pub fn new(code: &str, name: &str, description: &str) -> Self {
        Self {
            code: code.to_owned(),
            name: name.to_owned(),
            description: description.to_owned(),
            default: false,
        }
    }

    pub fn default_tab(mut self) -> Self {
        self.default = true;
        self
    }
}
