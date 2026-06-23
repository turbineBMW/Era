#[derive(Debug, Clone, PartialEq, Eq, glib::Variant)]
pub struct EventPropertiesPreset {
    pub name: String,
    pub description: String,
    pub location: String,
    pub conference: String,
    pub all_day: bool,
    pub start: String,
    pub end: String,
}

impl Default for EventPropertiesPreset {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            location: String::new(),
            conference: String::new(),
            all_day: true,
            start: "0001-01-01T00:00:00Z".to_string(),
            end: "0001-01-02T00:00:00Z".to_string(),
        }
    }
}
