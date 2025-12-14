use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidByte;

impl Display for InvalidByte {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid byte")?;
        Ok(())
    }
}
impl Error for InvalidByte {}

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
