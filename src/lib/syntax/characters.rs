use std::char::{decode_utf16, REPLACEMENT_CHARACTER};

pub fn characters_to_string<I: Iterator<Item = u16>>(i: I) -> String {
    decode_utf16(i)
        .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        .collect()
}

pub fn string_to_characters(s: String) -> Vec<u16> {
    s.encode_utf16().collect()
}
