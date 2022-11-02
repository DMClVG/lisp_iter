#![no_std]
use core::{fmt::Debug, iter::Chain, str::Chars};

#[derive(Clone)]
struct CharByteIter<T>
where
    T: Iterator<Item = char>,
{
    chars: T,
    byte: usize,
}

impl<'a, T> Iterator for CharByteIter<T>
where
    T: Iterator<Item = char>,
{
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.chars.next()?;
        let ret = Some((self.byte, c));
        self.byte += c.len_utf8();
        return ret;
    }
}

///
/// Iterator over a lisp expression provided by the input.
/// 
/// [`LispIter::next`] returns an [`Atom`]
/// 
#[derive(Clone)]
pub struct LispIter<'s> {
    pub input: &'s str,
    chars: CharByteIter<Chain<Chars<'s>, core::option::IntoIter<char>>>,
}

impl<'s> LispIter<'s> {
    pub fn new(input: &'s str) -> LispIter<'s> {
        LispIter {
            input,
            chars: CharByteIter {
                chars: input.chars().chain(Some('\n')),
                byte: 0,
            },
        }
    }
}

#[derive(Clone)]
pub enum Atom<'a> {
    /// Any unquoted word seperated by whitespaces or bound by a list.
    Identifier(&'a str),
    
    /// Any string between two " "
    /// 
    /// Note: quotes are unescaped i.e. \n \r and other escape sequences aren't taken into account.
    /// This is to prevent dynamic heap allocations.
    Quote(&'a str),

    /// Signed 64-bit integer.
    Integer(i64),

    /// 64-bit floating-point number.
    Float(f64),

    /// Anything between two ( )
    /// 
    /// Holds another [`LispIter`]
    List(LispIter<'a>),
}

/// Helper iterator convenient for iterating over a [`Atom::List`]'s contence.
/// 
/// Can be constructed by calling `.into_iterator()` on any [`Atom`]
///
pub struct AtomIter<'a> {
    atom: Option<Atom<'a>>,
}

impl<'a> AtomIter<'a> {
    pub fn new(atom: Atom<'a>) -> Self {
        Self { atom: Some(atom) }
    }
}

impl<'a> Iterator for AtomIter<'a> {
    type Item = Atom<'a>;

    /// Returns either the contence of [`Atom::List`] or itself if it is not an [`Atom::List`]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.atom {
            Some(Atom::List(iter)) => iter.next(),
            atom => atom.take(),
        }
    }
}

impl<'a> IntoIterator for Atom<'a> {
    type Item = Atom<'a>;
    type IntoIter = AtomIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AtomIter::new(self)
    }
}

impl Debug for Atom<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Identifier(arg0) => f.debug_tuple("Identifier").field(arg0).finish(),
            Self::Quote(arg0) => f.debug_tuple("Quote").field(arg0).finish(),
            Self::Integer(arg0) => f.debug_tuple("Integer").field(arg0).finish(),
            Self::Float(arg0) => f.debug_tuple("Float").field(arg0).finish(),
            Self::List(arg0) => f.debug_list().entries(arg0.clone()).finish(),
        }
    }
}

impl<'s> Iterator for LispIter<'s> {
    type Item = Atom<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let (start, c) = self.chars.by_ref().find(|(_, c)| !c.is_whitespace())?;
        match c {
            ';' => {
                self.chars.find(|(_, c)| *c == '\n');
                self.next()
            }
            '(' => {
                let mut popen = 0;
                let mut quoted = false;
                let mut commented = false;
                let (end, _) = self
                    .chars
                    .by_ref()
                    .find(|(_, c)| {
                        if popen == 0 && !quoted && !commented && *c == ')' {
                            return true;
                        } else {
                            match *c {
                                ';' => commented = true,
                                '\n' => commented = false,
                                '"' if !commented => quoted = !quoted,
                                '(' if !quoted && !commented => popen += 1,
                                ')' if !quoted && !commented => popen -= 1,
                                _ => {}
                            }
                            return false;
                        }
                    })
                    .unwrap_or_else(|| (self.input.len(), '\0')); // unclosed list

                Some(Atom::List(LispIter::new(
                    &self.input[start + '('.len_utf8()..end],
                )))
            }
            ')' => {
                unreachable!()
            }
            '"' => {
                let (end, _) = self
                    .chars
                    .by_ref()
                    .find(|(_, c)| *c == '"')
                    .unwrap_or_else(|| (self.input.len(), '\0')); // unclosed quote

                Some(Atom::Quote(&self.input[start + '"'.len_utf8()..end]))
            }
            ':' => {
                let (end, _) = self
                    .chars
                    .by_ref()
                    .find(|(_, c)| c.is_whitespace())
                    .unwrap();

                Some(Atom::Quote(&self.input[start + ':'.len_utf8()..end]))
            }
            '-' | '0'..='9' => {
                let (end, _) = self
                    .chars
                    .by_ref()
                    .find(|(_, c)| c.is_whitespace())
                    .unwrap();

                if let Ok(v) = self.input[start..end].parse() {
                    Some(Atom::Integer(v))
                } else if let Ok(v) = self.input[start..end].parse() {
                    Some(Atom::Float(v))
                } else {
                    Some(Atom::Identifier(&self.input[start..end])) // fallback
                }
            }
            _ => {
                let (end, _) = self
                    .chars
                    .by_ref()
                    .find(|(_, c)| c.is_whitespace())
                    .unwrap();
                Some(Atom::Identifier(&self.input[start..end]))
            }
        }
    }
}