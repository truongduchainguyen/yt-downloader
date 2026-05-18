#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle(String),
    Downloading(String),
    Error(String),
}
