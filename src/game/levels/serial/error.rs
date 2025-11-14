use crate::game::levels::serial::BindArgs;
use crate::game::levels::serial::kdl_utils::{DisplayValueType, KdlValueType};
use kdl::{KdlNode, NodeKey};
use miette::{ByteOffset, Diagnostic, LabeledSpan, Severity, SourceCode, SourceOffset, SourceSpan};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct KdlBindError {
    /// String to be displayed when this error is printed out.
    pub display: Option<String>,
    /// Source code that the error was taken from.
    pub source: Arc<String>,
    pub diagnostics: Vec<KdlBindDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct KdlBindDiagnostic {
    pub source: Arc<String>,
    pub span: Option<SourceSpan>,
    pub message: Option<String>,
    pub label: Option<String>,
    pub help: Option<String>,
    pub severity: Severity,
}

impl KdlBindError {
    pub fn new(source: Arc<String>) -> Self {
        Self {
            display: None,
            source,
            diagnostics: vec![],
        }
    }

    pub fn is_failure(&self) -> bool {
        self.diagnostics
            .iter()
            .map(|d| d.severity)
            .max()
            .is_some_and(|s| s >= Severity::Error)
    }

    pub fn merge(to_merge: Vec<Self>) -> Self {
        if to_merge.is_empty() {
            return Self::new(Arc::new("".to_string()));
        }

        return Self {
            display: to_merge[0].display.clone(),
            source: to_merge[0].source.clone(),
            diagnostics: to_merge
                .into_iter()
                .flat_map(|e| e.diagnostics.into_iter())
                .collect(),
        };
    }
}

impl Display for KdlBindError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display: &str = self
            .display
            .as_ref()
            .map(|s| s.as_ref())
            .unwrap_or("Failed to bind KDL document to object structure");
        write!(f, "{}", display)
    }
}

impl Error for KdlBindError {}

impl Diagnostic for KdlBindError {
    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source)
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        Some(Box::new(
            self.diagnostics.iter().map(|d| d as &dyn Diagnostic),
        ))
    }
}

impl KdlBindDiagnostic {
    pub fn new(source: Arc<String>, severity: Severity) -> Self {
        Self {
            source,
            span: None,
            message: None,
            label: None,
            help: None,
            severity,
        }
    }
}

impl Display for KdlBindDiagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display: &str = self
            .message
            .as_ref()
            .map(|s| s.as_ref())
            .unwrap_or("KDL Bind Error");
        write!(f, "{}", display)
    }
}

impl Error for KdlBindDiagnostic {}

impl Diagnostic for KdlBindDiagnostic {
    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.as_ref().map(|s| Box::new(s) as Box<dyn Display>)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let label = self.label.clone().unwrap_or_else(|| "here".to_string());
        self.span
            .map(|span| LabeledSpan::new_with_span(Some(label), span))
            .map(|labeled_span| {
                Box::new(iter::once(labeled_span)) as Box<dyn Iterator<Item = LabeledSpan>>
            })
    }
}

pub trait BindArgsErrorExt {
    fn err(&self, message: String, span: Option<SourceSpan>) -> KdlBindError;

    fn missing_element(&self, name: impl Display) -> KdlBindError;

    fn no_children(&self, span: SourceSpan, name: impl Display) -> KdlBindError;

    fn no_entry(
        &self,
        key: impl Into<NodeKey>,
        span: SourceSpan,
        name: impl Display,
    ) -> KdlBindError;

    fn wrong_value_type(
        &self,
        key: impl Into<NodeKey>,
        actual_type: KdlValueType,
        acceptable_types: &[KdlValueType],
        span: SourceSpan,
        name: impl Display,
    ) -> KdlBindError;
}

impl BindArgsErrorExt for BindArgs<'_, '_> {
    fn err(&self, message: String, span: Option<SourceSpan>) -> KdlBindError {
        KdlBindError {
            diagnostics: vec![KdlBindDiagnostic {
                message: Some(message),
                span,
                ..KdlBindDiagnostic::new(self.source.clone(), Severity::Error)
            }],
            ..KdlBindError::new(self.source.clone())
        }
    }

    fn missing_element(&self, name: impl Display) -> KdlBindError {
        self.err(format!("Missing element '{}'", name), None)
    }

    fn no_children(&self, span: SourceSpan, name: impl Display) -> KdlBindError {
        self.err(format!("Element '{}' has no children", name), Some(span))
    }

    fn no_entry(
        &self,
        key: impl Into<NodeKey>,
        span: SourceSpan,
        name: impl Display,
    ) -> KdlBindError {
        let msg = match key.into() {
            NodeKey::Key(key) => format!("Element '{}' missing property '{}'", name, key),
            NodeKey::Index(index) => format!("Element '{}' missing argument {}", name, index),
        };
        self.err(msg, Some(span))
    }

    fn wrong_value_type(
        &self,
        key: impl Into<NodeKey>,
        actual_type: KdlValueType,
        acceptable_types: &[KdlValueType],
        span: SourceSpan,
        name: impl Display,
    ) -> KdlBindError {
        let msg = match key.into() {
            NodeKey::Key(key) => format!(
                "Element '{}' property '{}' has type {} but should have been {}",
                name,
                key,
                actual_type,
                acceptable_types.display_type()
            ),
            NodeKey::Index(index) => format!(
                "Element '{}' argument {} has type {} but should have been {}",
                name,
                index,
                actual_type,
                acceptable_types.display_type()
            ),
        };
        self.err(msg, Some(span))
    }
}
