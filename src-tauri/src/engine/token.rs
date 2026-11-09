use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Bracketed representation, e.g. `[AFFICHER]`, `['users']`, `[$name]`.
    pub text: String,
    /// Original source slice.
    pub raw: String,
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub message: String,
    pub from: usize,
    pub to: usize,
    pub severity: String,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, from: usize, to: usize) -> Self {
        Self {
            message: message.into(),
            from,
            to,
            severity: "error".to_string(),
        }
    }
}
