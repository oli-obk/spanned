use bstr::{ByteSlice, Utf8Error};
use color_eyre::{eyre::Context, Report, Result};
use std::{fmt::Display, num::NonZeroUsize, path::PathBuf, str::FromStr};

#[derive(Clone)]
pub struct Spanned<T> {
    pub span: Span,
    pub content: T,
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
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
    pub file: PathBuf,
    pub line_start: NonZeroUsize,
    pub line_end: NonZeroUsize,
    pub col_start: NonZeroUsize,
    pub col_end: NonZeroUsize,
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
            line_start: NonZeroUsize::MAX,
            line_end: NonZeroUsize::MAX,
            col_start: NonZeroUsize::MAX,
            col_end: NonZeroUsize::MAX,
        }
    }
}

impl Span {
    pub fn is_dummy(&self) -> bool {
        self == &Self::default()
    }
    #[track_caller]
    pub fn dec_col_end(mut self, amount: usize) -> Self {
        self.col_end = NonZeroUsize::new(self.col_end.get() - amount).unwrap();
        self
    }
    #[track_caller]
    pub fn inc_col_start(mut self, amount: usize) -> Self {
        self.col_start = self.col_start.checked_add(amount).unwrap();
        self
    }
    #[track_caller]
    pub fn set_col_end_relative_to_start(mut self, amount: usize) -> Self {
        let new = self.col_start.checked_add(amount).unwrap();
        assert!(new <= self.col_end, "{self} new end: {new}");
        self.col_end = new;
        self
    }
    pub fn shrink_to_end(self) -> Span {
        Self {
            line_start: self.line_end,
            col_start: self.col_end,
            ..self
        }
    }

    pub fn shrink_to_start(self) -> Span {
        Self {
            line_end: self.line_start,
            col_end: self.col_start,
            ..self
        }
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
        let span = self.span.clone().dec_col_end(b.chars().count());
        let a = Spanned { span, content: a };
        let span = self
            .span
            .clone()
            .inc_col_start(a.content.chars().count() + 1);
        let b = Spanned { span, content: b };
        Some((a, b))
    }

    pub fn take_while(&self, delimiter: impl Fn(char) -> bool) -> Option<(Self, Self)> {
        let pos = self.content.find(|c| !delimiter(c))?;
        Some(self.split_at(pos))
    }

    pub fn split_at(&self, pos: usize) -> (Self, Self) {
        let (a, b) = self.content.split_at(pos);
        let n = a.chars().count();
        let span = self.span.clone().set_col_end_relative_to_start(n);
        let a = Spanned { span, content: a };
        let span = self.span.clone().inc_col_start(n);
        let b = Spanned { span, content: b };
        (a, b)
    }

    pub fn trim_end(&self) -> Self {
        let content = self.content.trim_end();
        let n = self.content[content.len()..].chars().count();
        let span = self.span.clone().dec_col_end(n);
        Self { content, span }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn strip_prefix(&self, prefix: &str) -> Option<Self> {
        let content = self.content.strip_prefix(prefix)?;
        let span = self.span.clone().inc_col_start(prefix.chars().count());
        Some(Self { content, span })
    }

    pub fn strip_suffix(&self, suffix: &str) -> Option<Self> {
        let content = self.content.strip_suffix(suffix)?;
        let span = self.span.clone().dec_col_end(suffix.chars().count());
        Some(Self { span, content })
    }

    pub fn trim_start(&self) -> Self {
        let content = self.content.trim_start();
        let n = self.content[..(self.content.len() - content.len())]
            .chars()
            .count();
        let span = self.span.clone().inc_col_start(n);
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

    pub fn chars(&self) -> impl Iterator<Item = Spanned<char>> + '_ {
        self.content.chars().enumerate().map(move |(i, c)| {
            Spanned::new(c, self.span.clone().inc_col_start(i).shrink_to_start())
        })
    }
}

impl<'a> Spanned<&'a [u8]> {
    pub fn strip_prefix(&self, prefix: &[u8]) -> Option<Self> {
        let content = self.content.strip_prefix(prefix)?;
        let span = self.span.clone().inc_col_start(prefix.chars().count());
        Some(Self { span, content })
    }

    pub fn split_once_str(&self, splitter: &str) -> Option<(Self, Self)> {
        let (a, b) = self.content.split_once_str(splitter)?;
        Some((
            Self {
                content: a,
                span: self
                    .span
                    .clone()
                    .set_col_end_relative_to_start(a.chars().count()),
            },
            Self {
                content: b,
                span: self
                    .span
                    .clone()
                    .inc_col_start(a.chars().count() + splitter.chars().count()),
            },
        ))
    }

    pub fn to_str(self) -> Result<Spanned<&'a str>, Spanned<Utf8Error>> {
        let span = self.span;
        match self.content.to_str() {
            Ok(content) => Ok(Spanned { content, span }),
            Err(err) => Err(Spanned { content: err, span }),
        }
    }
}

impl<T> Spanned<T> {
    pub fn new(content: T, span: Span) -> Self {
        Self { content, span }
    }
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

    pub fn span(&self) -> Span {
        self.span.clone()
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

    pub fn line(&self) -> NonZeroUsize {
        self.span.line_start
    }
}

impl Spanned<String> {
    pub fn read_from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let path_str = path.display().to_string();
        let content = std::fs::read_to_string(&path).with_context(|| path_str)?;
        let mut len = 0;
        let lines = content
            .lines()
            .inspect(|line| len = line.chars().count())
            .count();
        let span = Span {
            file: path,
            line_start: NonZeroUsize::new(1).unwrap(),
            line_end: NonZeroUsize::new(lines).unwrap(),
            col_start: NonZeroUsize::new(1).unwrap(),
            col_end: NonZeroUsize::new(len + 1).unwrap(),
        };
        Ok(Self { span, content })
    }
}

impl<T: AsRef<str>> Spanned<T> {
    /// Split up the string into lines
    pub fn lines(&self) -> impl Iterator<Item = Spanned<&str>> {
        assert_eq!(self.span.col_start.get(), 1);
        self.content
            .as_ref()
            .lines()
            .enumerate()
            .map(move |(i, content)| {
                let mut span = self.span.clone();
                span.line_start = span.line_start.checked_add(i).unwrap();
                span.line_end = span.line_start;
                span.col_end = NonZeroUsize::new(content.chars().count() + 1).unwrap();
                Spanned { content, span }
            })
    }
}
