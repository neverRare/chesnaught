use std::{
    cmp::Ordering,
    error::Error,
    fmt::{self, Display, Formatter},
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

pub const MEBIBYTES: usize = 1024 * 1024;

fn strip_prefix_token_untrimmed<'a>(src: &'a str, search: &str) -> Option<&'a str> {
    src.strip_prefix(search)
        .filter(|src| src.chars().next().is_none_or(<char>::is_whitespace))
}
pub fn starts_with_token(src: &str, search: &str) -> bool {
    strip_prefix_token_untrimmed(src, search).is_some()
}
pub fn strip_prefix_token<'a>(src: &'a str, search: &str) -> Option<&'a str> {
    strip_prefix_token_untrimmed(src, search).map(<str>::trim_start)
}
pub fn find_token(src: &str, search: &str) -> Option<usize> {
    src.match_indices(search).map(|(i, _)| i).find(|i| {
        src[(i + search.len())..]
            .chars()
            .next()
            .is_none_or(<char>::is_whitespace)
            && src[..*i]
                .chars()
                .next_back()
                .is_none_or(<char>::is_whitespace)
    })
}
pub fn split_by_token<'a>(src: &'a str, search: &str) -> Option<(&'a str, &'a str)> {
    find_token(src, search).map(|i| (src[..i].trim_end(), src[(i + search.len())..].trim_start()))
}
pub fn extract_prefix_token(src: &str) -> &str {
    match src.find(<char>::is_whitespace) {
        Some(i) => &src[..i],
        None => src,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompoundI8(i8);
impl CompoundI8 {
    pub fn new(left: i8, right: i8) -> Self {
        debug_assert!(left < 8);
        debug_assert!(left >= -8);
        debug_assert!(right < 8);
        debug_assert!(right >= -8);
        CompoundI8((left << 4) | (right & 0b_1111))
    }
    pub fn left(self) -> i8 {
        self.0 >> 4
    }
    pub fn right(self) -> i8 {
        (self.0 << 4) >> 4
    }
}
impl Default for CompoundI8 {
    fn default() -> Self {
        CompoundI8::new(0, 0)
    }
}
impl PartialOrd for CompoundI8 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CompoundI8 {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&(self.left(), self.right()), &(other.left(), other.right()))
    }
}
impl Neg for CompoundI8 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        CompoundI8::new(-self.left(), -self.right())
    }
}
impl Add<CompoundI8> for CompoundI8 {
    type Output = Self;

    fn add(self, rhs: CompoundI8) -> Self::Output {
        CompoundI8::new(self.left() + rhs.left(), self.right() + rhs.right())
    }
}
impl AddAssign<CompoundI8> for CompoundI8 {
    fn add_assign(&mut self, rhs: CompoundI8) {
        *self = CompoundI8::new(self.left() + rhs.left(), self.right() + rhs.right());
    }
}
impl Sub<CompoundI8> for CompoundI8 {
    type Output = Self;

    fn sub(self, rhs: CompoundI8) -> Self::Output {
        CompoundI8::new(self.left() - rhs.left(), self.right() - rhs.right())
    }
}
impl SubAssign<CompoundI8> for CompoundI8 {
    fn sub_assign(&mut self, rhs: CompoundI8) {
        *self = CompoundI8::new(self.left() - rhs.left(), self.right() - rhs.right());
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidByte;

impl Display for InvalidByte {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid byte")?;
        Ok(())
    }
}
impl Error for InvalidByte {}
