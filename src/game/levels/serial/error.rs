use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LevelError {
    pub input: Arc<String>,
    pub diagnostics: Vec<LevelDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct LevelDiagnostic {

}
