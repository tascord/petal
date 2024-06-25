use std::{fmt::Display, rc::Rc};

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceOffset, SourceSpan};

// Path, Content
pub type Hydrator = (String, Rc<String>);

#[derive(Debug)]
pub struct Error {
    pub during_process: String,
    pub error: String,
    pub hint: Option<String>,

    pub source: NamedSource<String>,
    pub source_path: String,
    pub source_code: String,

    pub position: (usize, usize),
    pub length: usize,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing: {}", self.during_process)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Diagnostic for Error {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(format!("Error parsing: {}", self.during_process)))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.hint
            .clone()
            .map(|hint| Box::new(hint) as Box<dyn std::fmt::Display>)
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        Some(Box::new(
            [LabeledSpan::new_primary_with_span(
                Some(self.error.clone()),
                SourceSpan::new(
                    SourceOffset::from_location(
                        self.source_code.to_string(),
                        self.position.0,
                        self.position.1,
                    ),
                    self.length,
                ),
            )]
            .into_iter(),
        ))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        None
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        None
    }
}

pub fn make_source(h: &Hydrator) -> NamedSource<String> {
    NamedSource::new(h.0.clone(), h.1.to_string())
}

macro_rules! partial {
    ($during:expr, $label:expr, $hint:expr, $span:expr, $h:expr) => {
        Error {
            during_process: format!("Failed while {}", $during),
            error: $label.to_string(),
            hint: Some($hint.to_string()),

            source: crate::errors::make_source(&$h),
            source_path: $h.0.clone(),
            source_code: $h.1.to_string(),

            position: $span.start_pos().line_col(),
            length: $span.end() - $span.start(),
        }
    };
    ($during:expr, $label:expr, $span:expr, $h:expr) => {
        Error {
            during_process: format!("Failed while {}", $during),
            error: $label.to_string(),
            hint: None,

            source: crate::errors::make_source(&$h),
            source_path: $h.0.clone(),
            source_code: $h.1.to_string(),

            position: $span.start_pos().line_col(),
            length: $span.end() - $span.start(),
        }
    };
}
