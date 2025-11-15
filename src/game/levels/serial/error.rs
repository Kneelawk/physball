#![allow(dead_code)]

use crate::game::levels::serial::kdl_utils::{DisplayValueType, KdlValueType};
use kdl::NodeKey;
use miette::{Diagnostic, LabeledSpan, Severity, SourceCode, SourceSpan};
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

        Self {
            display: to_merge[0].display.clone(),
            source: to_merge[0].source.clone(),
            diagnostics: to_merge
                .into_iter()
                .flat_map(|e| e.diagnostics.into_iter())
                .collect(),
        }
    }

    pub fn merge_two(self, other: Self) -> Self {
        Self {
            display: self.display,
            source: self.source,
            diagnostics: [self.diagnostics, other.diagnostics].concat(),
        }
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

pub trait MergeKdlBindError {
    type MergeResult;

    fn merge(self) -> Self::MergeResult;
}

macro_rules! __vars {
    (@res $($tt:tt)*) => {
        $($tt)*
    };
    ($first:ident @res $($tt:tt)*) => {
        __vars!(@res ($first, $($tt)*))
    };
    ($first:ident, $second:ident @res $($tt:tt)*) => {
        __vars!($second @res ($first, $($tt)*))
    };
    ($first:ident, $($ident:ident),+ @res $($tt:tt)*) => {
        __vars!($($ident),+ @res ($first, $($tt)*))
    };
    ($first:ident, $second:ident, $($ident:ident),+) => {
        __vars!($($ident),+ @res ($first, $second))
    };
}

macro_rules! __merges {
    (@res $($tt:tt)*) => {
        $($tt)*
    };
    ($first:ident @res $($tt:tt)*) => {
        __merges!(@res MergeKdlBindError::merge(($first, $($tt)*)))
    };
    ($first:ident, $($ident:ident),+ @res $($tt:tt)*) => {
        __merges!($($ident),+ @res MergeKdlBindError::merge(($first, $($tt)*)))
    };
    ($first:ident, $second:ident, $($ident:ident),+) => {
        __merges!($($ident),+ @res MergeKdlBindError::merge(($first, $second)))
    };
}

macro_rules! impl_result_merge {
    ($first:ident:$zero:ident, $second:ident:$one:ident, $($ident:ident:$num:ident),+) => {
        impl<$first, $second, $($ident),+> MergeKdlBindError for (Result<$first, KdlBindError>, Result<$second, KdlBindError>, $(Result<$ident, KdlBindError>),+) {
            type MergeResult = Result<($first, $second, $($ident),+), KdlBindError>;

            fn merge(self) -> Self::MergeResult {
                let ($zero, $one, $($num),+) = self;
                let __vars!($zero, $one, $($num),+) = __merges!($zero, $one, $($num),+)?;
                Ok(($zero, $one, $($num),+))
            }
        }
    };
    ($first:ident:$zero:ident, $second:ident:$one:ident) => {
        impl<$first, $second> MergeKdlBindError for (Result<$first, KdlBindError>, Result<$second, KdlBindError>) {
            type MergeResult = Result<($first, $second), KdlBindError>;

            #[inline]
            fn merge(self) -> Self::MergeResult {
                let ($zero, $one) = self;
                match $zero {
                    Ok(res) => match $one {
                        Ok(res1) => Ok((res, res1)),
                        Err(err) => Err(err),
                    }
                    Err(err) => match $one {
                        Ok(_res) => Err(err),
                        Err(err1) => Err(err1.merge_two(err))
                    }
                }
            }
        }
    };
}

impl_result_merge!(T0:t0, T1:t1);
impl_result_merge!(T0:t0, T1:t1, T2:t2);
impl_result_merge!(T0:t0, T1:t1, T2:t2, T3:t3);
impl_result_merge!(T0:t0, T1:t1, T2:t2, T3:t3, T4:t4);

impl<T> MergeKdlBindError for Vec<Result<T, KdlBindError>> {
    type MergeResult = Result<Vec<T>, KdlBindError>;

    fn merge(self) -> Self::MergeResult {
        if self.is_empty() {
            return Ok(vec![]);
        }

        let mut display = None;
        let mut source = None;
        let mut diagnostics = vec![];
        let mut results = vec![];

        for res in self {
            match res {
                Ok(t) => {
                    results.push(t);
                }
                Err(mut err) => {
                    diagnostics.append(&mut err.diagnostics);
                    if display.is_none() {
                        display = Some(err.display);
                    }
                    if source.is_none() {
                        source = Some(err.source);
                    }
                }
            }
        }

        if diagnostics.is_empty() {
            Ok(results)
        } else {
            Err(KdlBindError {
                display: display.unwrap(),
                source: source.unwrap(),
                diagnostics,
            })
        }
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

pub trait BindErrorExt {
    fn err(&self, message: String, span: Option<SourceSpan>) -> KdlBindError;

    fn missing_element(&self, name: impl Display) -> KdlBindError;

    fn no_children(&self, span: SourceSpan) -> KdlBindError;

    fn no_entry(&self, key: impl Into<NodeKey>, span: SourceSpan) -> KdlBindError;

    fn wrong_value_type(
        &self,
        actual_type: KdlValueType,
        acceptable_types: &[KdlValueType],
        span: SourceSpan,
    ) -> KdlBindError;

    fn parse_error(&self, msg: impl Display, span: SourceSpan) -> KdlBindError;

    fn not_a_variant<T: Display>(
        &self,
        provided: impl Display,
        variants: &[T],
        span: SourceSpan,
    ) -> KdlBindError;
}

impl BindErrorExt for Arc<String> {
    fn err(&self, message: String, span: Option<SourceSpan>) -> KdlBindError {
        KdlBindError {
            diagnostics: vec![KdlBindDiagnostic {
                message: Some(message),
                span,
                ..KdlBindDiagnostic::new(self.clone(), Severity::Error)
            }],
            ..KdlBindError::new(self.clone())
        }
    }

    fn missing_element(&self, name: impl Display) -> KdlBindError {
        self.err(format!("Missing element '{}'", name), None)
    }

    fn no_children(&self, span: SourceSpan) -> KdlBindError {
        self.err("Element has no children".to_string(), Some(span))
    }

    fn no_entry(&self, key: impl Into<NodeKey>, span: SourceSpan) -> KdlBindError {
        let msg = match key.into() {
            NodeKey::Key(key) => format!("Element missing property '{}'", key),
            NodeKey::Index(index) => format!("Element missing argument {}", index),
        };
        self.err(msg, Some(span))
    }

    fn wrong_value_type(
        &self,
        actual_type: KdlValueType,
        acceptable_types: &[KdlValueType],
        span: SourceSpan,
    ) -> KdlBindError {
        self.err(
            format!(
                "Element value has type {} but should have been {}",
                actual_type,
                acceptable_types.display_type()
            ),
            Some(span),
        )
    }

    fn parse_error(&self, msg: impl Display, span: SourceSpan) -> KdlBindError {
        self.err(format!("Element value parsing error: {}", msg), Some(span))
    }

    fn not_a_variant<T: Display>(
        &self,
        provided: impl Display,
        variants: &[T],
        span: SourceSpan,
    ) -> KdlBindError {
        let variants_str = match variants.len() {
            1 => {
                format!("the allowed variant is '{}'", variants[0])
            }
            2 => {
                format!(
                    "the allowed variants are '{}' and '{}'",
                    variants[0], variants[1]
                )
            }
            _ => {
                let mut str = "the allowed variants are ".to_string();
                for (i, var) in variants.iter().enumerate() {
                    if i > 0 {
                        str += ", ";
                    }
                    if i > variants.len() - 1 {
                        str += "and ";
                    }
                    str += &format!("'{}'", var);
                }
                str
            }
        };

        self.err(
            format!("Invalid variant provided: '{}', {}", provided, variants_str),
            Some(span),
        )
    }
}
