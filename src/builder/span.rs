use core::{fmt, ops::Index};
use std::{
    collections::{HashMap, hash_map::Entry},
    path::{Path, PathBuf},
};

pub fn span(file_ctx: &str, line_offset: u32) -> LocatedSpan {
    unsafe { LocatedSpan::new_from_raw_offset(0, 1 + line_offset, file_ctx, ()) }
}
pub type LocatedSpan<'a> = nom_locate::LocatedSpan<&'a str>;

#[derive(Debug, Clone, Default, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    /// The line number of the fragment relatively to the input of the parser. It starts at line 1.
    pub line_num: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub start: usize,
    pub line_num: u32,
}

impl Pos {
    pub fn new(i: LocatedSpan) -> Option<Self> {
        if let Ok((_, pos)) = nom_locate::position::<LocatedSpan, ()>(i) {
            Some(pos.into())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileId {
    Include { path: PathBuf },
    Section { path: PathBuf, section: String },
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileId::Include { path } => write!(f, "File \"{}\"", path.display()),
            FileId::Section { path, section } => {
                write!(f, "File \"{}\", section {section}", path.display())
            }
        }
    }
}

impl FileId {
    pub fn path(&self) -> &Path {
        match self {
            FileId::Include { path } => path,
            FileId::Section { path, section: _ } => path,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EndReason<'a> {
    Include {
        file_name: &'a Path,
        section: Option<String>,
    },
    End,
}

#[derive(Debug, Clone, Copy)]
pub struct ParsedId(pub usize);

#[derive(Debug)]
pub struct FileStorage<Parsed> {
    pub file: Vec<String>,
    pub parsed: Vec<(FileId, Parsed)>,
    pub id2idx: HashMap<FileId, ParsedId>,
}
impl<Parsed> Default for FileStorage<Parsed> {
    #[inline]
    fn default() -> Self {
        const CAP: usize = 4;
        Self {
            id2idx: HashMap::with_capacity(CAP),
            file: Vec::with_capacity(CAP),
            parsed: Vec::with_capacity(CAP),
        }
    }
}
impl<Parsed: Default> FileStorage<Parsed> {
    pub fn existed(&self, file_id: &FileId) -> Option<ParsedId> {
        log::debug!("load {}", file_id);
        self.id2idx.get(file_id).copied()
    }
    pub fn new_file(&mut self, file_id: FileId) -> ParsedId {
        match self.id2idx.entry(file_id) {
            Entry::Occupied(occupied) => {
                log::warn!("already loaded {:?}", occupied.key());
                *occupied.get()
            }
            Entry::Vacant(vacant_entry) => {
                let files_num = self.file.len();
                self.file.push(String::new());
                self.parsed
                    .push((vacant_entry.key().clone(), Parsed::default()));
                *vacant_entry.insert(ParsedId(files_num))
            }
        }
    }
    pub fn update_ctx(&mut self, parsed_id: &ParsedId, file_ctx: String, parsed: Parsed) {
        self.file[parsed_id.0] = file_ctx;
        self.parsed[parsed_id.0].1 = parsed;
    }
}

impl Index<&Span> for str {
    type Output = str;
    #[inline]
    fn index(&self, index: &Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}

impl Index<&Span> for String {
    type Output = str;
    #[inline]
    fn index(&self, index: &Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}

impl From<LocatedSpan<'_>> for Span {
    #[inline]
    fn from(s: LocatedSpan<'_>) -> Self {
        let start = s.location_offset();
        Self {
            start,
            end: start + s.fragment().len(),
            line_num: s.location_line(),
        }
    }
}

impl From<LocatedSpan<'_>> for Pos {
    #[inline]
    fn from(s: LocatedSpan<'_>) -> Self {
        let start = s.location_offset();
        Self {
            start,
            line_num: s.location_line(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::{
        IResult,
        bytes::complete::{tag, take_until},
    };

    #[derive(Debug, Default)]
    struct Token {
        // pub pos: Pos,
        pub _foo: Span,
        pub _bar: Span,
    }

    fn parse_foobar(s: LocatedSpan) -> IResult<LocatedSpan, Token> {
        let (s, _) = take_until("foo")(s)?;
        // let (s, pos) = position(s)?;
        let (s, foo) = tag("foo")(s)?;
        let (s, bar) = tag("bar")(s)?;
        Ok((
            s,
            Token {
                // pos: pos.into(),
                _foo: foo.into(),
                _bar: bar.into(),
            },
        ))
    }
    #[test]
    fn main() {
        let mut file_storage = FileStorage::default();
        let file_ctx = String::from("Lorem ipsum \n foobar");
        let parsed_id = file_storage.new_file(FileId::Include {
            path: "dummy.sp".into(),
        });
        let input = span(&file_ctx, 0);
        let output = parse_foobar(input).unwrap().1;
        file_storage.update_ctx(&parsed_id, file_ctx, output);
        let file = &file_storage.file[parsed_id.0];
        println!("{}", &file[&file_storage.parsed[parsed_id.0].1._foo]);
    }
}
