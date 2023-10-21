use color_eyre::{eyre::Context, Report, Result};
use std::{fmt::Display, path::PathBuf, str::FromStr};

#[derive(Clone)]
pub struct Spanned<T> {
    pub span: Span,
    pub content: T,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.span, f)?;
        write!(f, ": ")?;
        self.content.fmt(f)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Span {
    file: PathBuf,
    line_start: usize,
    line_end: usize,
    col_start: usize,
    col_end: usize,
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl Default for Span {
    fn default() -> Self {
        Self {
            file: PathBuf::new(),
            line_start: 0,
            line_end: 0,
            col_start: 0,
            col_end: 0,
        }
    }
}

impl Span {
    pub fn is_dummy(&self) -> bool {
        self == &Self::default()
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_dummy() {
            return write!(f, "DUMMY_SPAN");
        }
        let Self {
            file,
            line_start,
            line_end,
            col_start,
            col_end,
        } = self;
        let file = file.display();
        write!(f, "{file}:{line_start}:{col_start} {line_end}:{col_end}")
    }
}

impl Spanned<&str> {
    pub fn split_once(&self, delimiter: &str) -> Option<(Self, Self)> {
        let (a, b) = self.content.split_once(delimiter)?;
        let mut span = self.span.clone();
        span.col_end -= b.chars().count();
        let a = Spanned { span, content: a };
        let mut span = self.span.clone();
        span.col_start += a.content.chars().count() + 1;
        let b = Spanned { span, content: b };
        Some((a, b))
    }

    pub fn take_while(&self, delimiter: impl Fn(char) -> bool) -> Option<(Self, Self)> {
        let pos = self.content.find(|c| !delimiter(c))?;
        Some(self.split_at(pos))
    }

    pub fn split_at(&self, pos: usize) -> (Self, Self) {
        let (a, b) = self.content.split_at(pos);
        let mut span = self.span.clone();
        span.col_end -= b.chars().count();
        let a = Spanned { span, content: a };
        let mut span = self.span.clone();
        span.col_start += a.content.chars().count() + 1;
        let b = Spanned { span, content: b };
        (a, b)
    }

    pub fn trim_end(&self) -> Self {
        let content = self.content.trim_end();
        let n = self.content[content.len()..].chars().count();
        let mut span = self.span.clone();
        span.col_end -= n;
        Self { content, span }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn strip_prefix(&self, prefix: &str) -> Option<Self> {
        let content = self.content.strip_prefix(prefix)?;
        let n = self.content[..(self.content.len() - content.len())]
            .chars()
            .count();
        let mut span = self.span.clone();
        span.col_start += n;
        Some(Self { content, span })
    }

    pub fn trim_start(&self) -> Self {
        let content = self.content.trim_start();
        let n = self.content[..(self.content.len() - content.len())]
            .chars()
            .count();
        let mut span = self.span.clone();
        span.col_start += n;
        Self { content, span }
    }

    pub fn trim(&self) -> Self {
        self.trim_start().trim_end()
    }

    pub fn starts_with(&self, pat: &str) -> bool {
        self.content.starts_with(pat)
    }

    pub fn parse<T: FromStr>(self) -> Result<Spanned<T>>
    where
        T::Err: Into<Report>,
    {
        let content = self
            .content
            .parse()
            .map_err(Into::into)
            .with_context(|| self.span.clone())?;
        Ok(Spanned {
            span: self.span,
            content,
        })
    }
}

impl<T> Spanned<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        let Spanned { content, span } = self;
        let content = f(content);
        Spanned { content, span }
    }

    pub fn dummy(content: T) -> Self {
        Self {
            span: Span::default(),
            content,
        }
    }

    pub fn as_ref<U: ?Sized>(&self) -> Spanned<&U>
    where
        T: AsRef<U>,
    {
        Spanned {
            span: self.span.clone(),
            content: self.content.as_ref(),
        }
    }
}

impl Spanned<String> {
    pub fn read_from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let path_str = path.display().to_string();
        let content = std::fs::read_to_string(&path).with_context(|| path_str)?;
        let span = Span {
            file: path,
            line_start: 1,
            line_end: content.lines().count() + 1,
            col_start: 1,
            col_end: 0,
        };
        Ok(Self { span, content })
    }
}

impl Spanned<&str> {
    /// Split up the string into lines
    pub fn lines<'a>(&'a self) -> impl Iterator<Item = Spanned<&'a str>> {
        assert_eq!(self.span.col_start, 1);
        self.content.lines().enumerate().map(move |(i, content)| {
            let mut span = self.span.clone();
            span.line_start += i;
            span.line_end = span.line_start;
            span.col_end = content.chars().count();
            Spanned { content, span }
        })
    }
}
