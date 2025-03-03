use super::builder::span::{FileId, LocatedSpan, Pos, Span};

use anstyle::Style;
use core::{cell::LazyCell, fmt};
use indexmap::IndexMap;
use nom::error::ErrorKind;
use std::path::PathBuf;

impl From<nom::Err<nom::error::Error<LocatedSpan<'_>>>> for ParseError {
    #[inline]
    fn from(e: nom::Err<nom::error::Error<LocatedSpan<'_>>>) -> Self {
        match e {
            nom::Err::Incomplete(_) => ParseErrorInner::Nom(None).with(None),
            nom::Err::Failure(e) | nom::Err::Error(e) => {
                ParseErrorInner::Nom(Some(e.code)).record(e.input)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseErrorInner {
    #[error("Incomplete")]
    IO(#[from] std::io::Error),
    #[error("Can NOT find section [{section}] in file {path}")]
    NoLibSection { path: PathBuf, section: String },
    /// Nom Error
    #[error("Syntax error")]
    Nom(Option<ErrorKind>),
    /// something else
    #[error("{0:?}")]
    Unknown(Span),
    #[error("Circular definition")]
    CircularDefinition(IndexMap<FileId, Option<Pos>>, usize),
}

impl ParseErrorInner {
    pub fn record(self, i: LocatedSpan) -> ParseError {
        ParseError {
            pos: Pos::new(i),
            err: self,
        }
    }
    pub fn with(self, pos: Option<Pos>) -> ParseError {
        ParseError { pos, err: self }
    }
}
#[derive(Debug, Clone, Copy)]
struct Styles {
    msg: Style,
    typ: Style,
    err: Style,
}
const STYLES: LazyCell<Styles> = LazyCell::new(|| {
    use anstyle::{AnsiColor, Color};
    if colored::control::SHOULD_COLORIZE.should_colorize() {
        Styles {
            msg: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightMagenta))),
            typ: Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightMagenta)))
                .bold(),
            err: Style::new()
                .fg_color(Some(AnsiColor::BrightRed.into()))
                .bold(),
        }
    } else {
        Styles {
            msg: Style::new(),
            typ: Style::new(),
            err: Style::new(),
        }
    }
});

#[derive(Debug)]
pub struct ParseError {
    pub pos: Option<Pos>,
    pub err: ParseErrorInner,
}

impl ParseError {
    pub fn report(&self, has_err: &mut bool, file_id: &FileId, file: &str) {
        *has_err = true;
        struct ReportDisplay<'a> {
            err: &'a ParseError,
            file_id: &'a FileId,
            file: &'a str,
        }
        impl fmt::Display for ReportDisplay<'_> {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                use super::builder::Builder as _;
                let styles: Styles = *STYLES;
                write!(
                    f,
                    "\nFile {}\"{}\"{}",
                    styles.msg.render(),
                    self.file_id.path().display(),
                    styles.msg.render_reset()
                )?;
                if let Some(pos) = self.err.pos {
                    write!(
                        f,
                        ", line {}{}{}",
                        styles.msg.render(),
                        pos.line_num,
                        styles.msg.render_reset()
                    )?;
                    let span = unsafe {
                        LocatedSpan::new_from_raw_offset(
                            pos.start,
                            pos.line_num,
                            &self.file[pos.start..],
                            (),
                        )
                    };
                    if let Ok(s) = core::str::from_utf8(span.get_line_beginning()) {
                        write!(f, "\n{s}\n")?;
                        for _ in 0..span.get_column() - 1 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}<-{}", styles.err.render(), styles.err.render_reset())?;
                    }
                }
                writeln!(f)?;
                match &self.err.err {
                    ParseErrorInner::IO(error) => {
                        writeln!(
                            f,
                            "{}Error{}: {}{error}{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                            styles.msg.render(),
                            styles.msg.render_reset()
                        )
                    }
                    ParseErrorInner::NoLibSection { path, section } => {
                        writeln!(
                            f,
                            "{}Error{}: {}Can NOT find section `{section}` in file \"{}\"{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                            styles.msg.render(),
                            path.display(),
                            styles.msg.render_reset()
                        )
                    }
                    ParseErrorInner::Nom(e) => {
                        write!(
                            f,
                            "{}ParserError{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                        )?;
                        if let Some(e) = e {
                            writeln!(
                                f,
                                ": {}{e:?}{}",
                                styles.msg.render(),
                                styles.msg.render_reset()
                            )
                        } else {
                            writeln!(f)
                        }
                    }
                    ParseErrorInner::Unknown(span) => {
                        writeln!(
                            f,
                            "{}SyntaxError{}: {}Unknwon command `{}`{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                            styles.msg.render(),
                            span.build(self.file),
                            styles.msg.render_reset()
                        )
                    }
                    ParseErrorInner::CircularDefinition(index_set, idx) => {
                        struct FileDisplay<'a>(&'a FileId, &'a Option<Pos>);
                        impl fmt::Display for FileDisplay<'_> {
                            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                                match self.0 {
                                    FileId::Include { path } => {
                                        write!(f, "File \"{}\"", path.display())?;
                                        if let Some(pos) = self.1 {
                                            write!(f, ", line {}", pos.line_num)?;
                                        }
                                        Ok(())
                                    }
                                    FileId::Section { path, section } => {
                                        write!(f, "File \"{}\"", path.display())?;
                                        if let Some(pos) = self.1 {
                                            write!(f, ", line {}", pos.line_num)?;
                                        }
                                        write!(f, ", section {section}")
                                    }
                                }
                            }
                        }
                        impl<'s> FileDisplay<'s> {
                            fn new(f: (&'s FileId, &'s Option<Pos>)) -> Self {
                                Self(f.0, f.1)
                            }
                        }
                        let circular_file = index_set.get_index(*idx).unwrap();
                        writeln!(
                            f,
                            "{}CircularDefinition{}: {}Detect circular definition in {}{}",
                            styles.typ.render(),
                            styles.typ.render_reset(),
                            styles.msg.render(),
                            FileDisplay::new(circular_file),
                            styles.msg.render_reset()
                        )?;
                        for (i, file) in index_set.iter().enumerate() {
                            if *idx == i {
                                writeln!(
                                    f,
                                    "{} * {}{}\n     ↓",
                                    styles.err.render(),
                                    FileDisplay::new(file),
                                    styles.err.render_reset()
                                )?;
                            } else {
                                writeln!(f, "   {}\n     ↓", FileDisplay::new(file))?;
                            }
                        }
                        writeln!(
                            f,
                            "{} * {}{}",
                            styles.err.render(),
                            FileDisplay::new(circular_file),
                            styles.err.render_reset()
                        )
                    }
                }
            }
        }
        log::error!(
            "{}",
            ReportDisplay {
                err: self,
                file_id,
                file
            }
        )
    }
}
