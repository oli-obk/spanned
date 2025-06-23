use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use annotate_snippets::{Level, Renderer, Snippet};

use crate::{Span, Spanned};

/// An error type that maintains multiple nested spans and ensures they all get printed together in one nice diagnostic message.
pub struct Error {
    data: Box<ErrorData<dyn std::error::Error>>,
}

impl Error {
    pub fn wrap<T: std::error::Error + 'static>(self, context: Spanned<T>) -> Self {
        Self {
            data: Box::new(ErrorData {
                span: context.span,
                source: Some(self),
                data: context.content,
            }),
        }
    }

    pub fn wrap_str<T: Display + 'static>(self, context: Spanned<T>) -> Self {
        Self {
            data: Box::new(ErrorData {
                span: context.span,
                source: Some(self),
                data: DisplayData(context.content),
            }),
        }
    }

    pub fn new<T: std::error::Error + 'static>(context: Spanned<T>) -> Self {
        Self {
            data: Box::new(ErrorData {
                span: context.span,
                source: None,
                data: context.content,
            }),
        }
    }

    #[track_caller]
    pub fn here<T: std::error::Error + 'static>(data: T) -> Self {
        Self {
            data: Box::new(ErrorData {
                span: Span::here(),
                source: None,
                data,
            }),
        }
    }

    #[track_caller]
    pub fn str<T: Display + 'static>(data: T) -> Self {
        Self {
            data: Box::new(ErrorData {
                span: Span::here(),
                source: None,
                data: DisplayData(data),
            }),
        }
    }

    fn sources(&self) -> SourceIter<'_> {
        SourceIter(self.data.source.as_ref())
    }
}

struct SourceIter<'a>(Option<&'a Error>);
impl<'a> Iterator for SourceIter<'a> {
    type Item = &'a Error;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.take()?;
        self.0 = next.data.source.as_ref();
        Some(next)
    }
}

struct DisplayData<T: ?Sized>(T);

impl<T: Display + ?Sized> std::error::Error for DisplayData<T> {}

impl<T: Display + ?Sized> Display for DisplayData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Display + ?Sized> Debug for DisplayData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: std::error::Error + 'static> From<Spanned<T>> for Error {
    fn from(value: Spanned<T>) -> Self {
        Self {
            data: Box::new(ErrorData {
                span: value.span,
                source: None,
                data: value.content,
            }),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.data
            .source
            .as_ref()
            .map(|e| e as &dyn std::error::Error)
    }
}

struct ErrorData<T: std::error::Error + ?Sized> {
    span: Span,
    source: Option<Error>,
    data: T,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut files = HashMap::new();
        files.insert(
            &self.data.span.file,
            (
                std::fs::read_to_string(&self.data.span.file).unwrap(),
                self.data.span.file.display().to_string(),
                vec![],
            ),
        );
        for e in self.sources() {
            let (_, _, list) = files.entry(&e.data.span.file).or_insert_with(|| {
                (
                    std::fs::read_to_string(&e.data.span.file).unwrap(),
                    e.data.span.file.display().to_string(),
                    vec![],
                )
            });
            list.push((e.data.span.bytes.clone(), e.data.data.to_string()))
        }

        let title = self.data.data.to_string();
        let (main_file, main_path, main_labels) = &files[&self.data.span.file];
        let message = Level::Error.title(&title).snippets(
            [Snippet::source(main_file)
                .origin(main_path)
                .fold(true)
                .annotation(Level::Error.span(self.data.span.bytes.clone()))
                .annotations(
                    main_labels
                        .iter()
                        .map(|(span, msg)| Level::Error.span(span.clone()).label(msg)),
                )]
            .into_iter()
            .chain(self.sources().filter_map(|e| {
                let (file, path, labels) = &files[&e.data.span.file];
                if path == main_path {
                    return None;
                }
                Some(
                    Snippet::source(file).origin(path).fold(true).annotations(
                        labels
                            .iter()
                            .map(|(span, msg)| Level::Error.span(span.clone()).label(msg)),
                    ),
                )
            })),
        );
        let renderer = if colored::control::SHOULD_COLORIZE.should_colorize() {
            Renderer::styled()
        } else {
            Renderer::plain()
        };
        let res = write!(f, "{}", renderer.render(message));
        res
    }
}
