pub trait Fragment: Sized {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>;
    fn from_json(fragment: &str) -> Result<Self, serde_json::Error>;
}
