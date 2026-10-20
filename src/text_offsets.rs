//! Offset conversions between GPUI's UTF-8 storage, Windows IME's UTF-16,
//! and the bullet string displayed by a masked input.
pub fn from_utf16(text: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    for (index, character) in text.char_indices() {
        if utf16 >= offset {
            return index;
        }
        utf16 += character.len_utf16();
    }
    text.len()
}

pub fn to_utf16(text: &str, offset: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < offset)
        .map(|(_, character)| character.len_utf16())
        .sum()
}

pub fn to_mask(text: &str, offset: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < offset)
        .count()
        * '•'.len_utf8()
}

pub fn from_mask(text: &str, offset: usize) -> usize {
    text.char_indices()
        .nth(offset / '•'.len_utf8())
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_source_ime_and_mask_positions_round_trip() {
        let text = "a中🔑e\u{301}";
        for (source, utf16, mask) in [
            (0, 0, 0),
            (1, 1, 3),
            (4, 2, 6),
            (8, 4, 9),
            (9, 5, 12),
            (11, 6, 15),
        ] {
            assert_eq!(to_utf16(text, source), utf16);
            assert_eq!(from_utf16(text, utf16), source);
            assert_eq!(to_mask(text, source), mask);
            assert_eq!(from_mask(text, mask), source);
        }
    }

    #[test]
    fn ime_offsets_clamp_at_end_and_never_split_surrogates() {
        assert_eq!(from_utf16("🔑", 1), 4);
        assert_eq!(from_utf16("🔑", 99), 4);
        assert_eq!(from_mask("中", 99), 3);
        assert_eq!(from_utf16("", 10), 0);
    }
}
