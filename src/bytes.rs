use ratatui::buffer::CellWidth as _;
use unicode_segmentation::UnicodeSegmentation as _;

const TAB_SIZE: usize = 4;

pub fn normalize_stdout(s: &[u8]) -> Vec<u8> {
    enum GraphemeKind {
        Text,
        Tab,
        LineBreak,
        Backspace,
        IgnoredControl,
    }

    let str = String::from_utf8_lossy(s).to_string();
    let mut chars = str.chars();
    let mut items = Vec::new();
    let mut visible = String::new();
    let mut visible_byte_ranges = Vec::new();

    while let Some(c) = chars.next() {
        let count = skip_ansi_escape_sequence(c, &mut chars.clone());
        if count > 0 {
            let mut sequence = String::new();
            sequence.push(c);
            for _ in 0..count {
                if let Some(ch) = chars.next() {
                    sequence.push(ch);
                }
            }
            items.push((sequence, None));
            continue;
        }

        let visible_index = visible_byte_ranges.len();
        let byte_start = visible.len();
        visible.push(c);
        let byte_end = visible.len();
        visible_byte_ranges.push((byte_start, byte_end));
        items.push((c.to_string(), Some(visible_index)));
    }

    let mut visible_to_grapheme = vec![0; visible_byte_ranges.len()];
    let mut graphemes = Vec::new();
    let mut visible_cursor = 0;
    for (grapheme_id, (byte_start, grapheme)) in visible.grapheme_indices(true).enumerate() {
        let byte_end = byte_start + grapheme.len();
        let first_visible = visible_cursor;
        while visible_cursor < visible_byte_ranges.len()
            && visible_byte_ranges[visible_cursor].0 < byte_end
        {
            visible_to_grapheme[visible_cursor] = grapheme_id;
            visible_cursor += 1;
        }

        if first_visible == visible_cursor {
            continue;
        }

        let last_visible = visible_cursor - 1;
        let mut grapheme_chars = grapheme.chars();
        let single_char = match (grapheme_chars.next(), grapheme_chars.next()) {
            (Some(c), None) => Some(c),
            _ => None,
        };
        let kind = match grapheme {
            "\t" => GraphemeKind::Tab,
            "\n" | "\r" | "\r\n" | "\u{000B}" | "\u{000C}" => GraphemeKind::LineBreak,
            "\u{0008}" => GraphemeKind::Backspace,
            _ if matches!(
                single_char,
                Some('\u{0000}'..='\u{0007}' | '\u{000E}'..='\u{001F}' | '\u{007F}')
            ) =>
            {
                GraphemeKind::IgnoredControl
            }
            _ => GraphemeKind::Text,
        };
        let width = if let GraphemeKind::Text = kind {
            grapheme.cell_width() as usize
        } else {
            0
        };
        graphemes.push((first_visible, last_visible, width, kind));
    }

    let mut b = String::with_capacity(str.len() * TAB_SIZE);
    let mut width = 0;
    for (item, visible_index) in items {
        let Some(visible_index) = visible_index else {
            b.push_str(&item);
            continue;
        };

        let (first_visible, last_visible, grapheme_width, ref kind) =
            graphemes[visible_to_grapheme[visible_index]];

        if let GraphemeKind::Tab = kind {
            if visible_index == first_visible {
                let spaces = TAB_SIZE - (width % TAB_SIZE);
                b.extend(std::iter::repeat_n(' ', spaces));
                width += spaces;
            }
            continue;
        }

        b.push_str(&item);
        if visible_index == last_visible {
            match kind {
                GraphemeKind::LineBreak => width = 0,
                GraphemeKind::Backspace => width = width.saturating_sub(1),
                GraphemeKind::IgnoredControl => {}
                GraphemeKind::Text => width += grapheme_width,
                GraphemeKind::Tab => {}
            }
        }
    }

    b.into_bytes()
}

// Based on https://github.com/mgeisler/textwrap/blob/63970361d1d653ec8715acb931c3c109750d4a57/src/core.rs
/// The CSI or “Control Sequence Introducer” introduces an ANSI escape
/// sequence. This is typically used for colored text and will be
/// ignored when computing the text width.
const CSI: (char, char) = ('\x1b', '[');
/// The final bytes of an ANSI escape sequence must be in this range.
const ANSI_FINAL_BYTE: std::ops::RangeInclusive<char> = '\x40'..='\x7e';
/// Skip ANSI escape sequences.
///
/// The `ch` is the current `char`, the `chars` provide the following
/// characters. The `chars` will be modified if `ch` is the start of
/// an ANSI escape sequence.
///
/// Returns `usize` the count of skipped characters
fn skip_ansi_escape_sequence<I: Iterator<Item = char>>(ch: char, chars: &mut I) -> usize {
    let mut count = 0;
    if ch != CSI.0 {
        return 0; // Nothing to skip here.
    }

    let next = chars.next();
    count += 1;
    if next == Some(CSI.1) {
        // We have found the start of an ANSI escape code, typically
        // used for colored terminal text. We skip until we find a
        // "final byte" in the range 0x40–0x7E.
        for ch in chars {
            count += 1;
            if ANSI_FINAL_BYTE.contains(&ch) {
                break;
            }
        }
    } else if next == Some(']') {
        // We have found the start of an Operating System Command,
        // which extends until the next sequence "\x1b\\" (the String
        // Terminator sequence) or the BEL character. The BEL
        // character is non-standard, but it is still used quite
        // often, for example, by GNU ls.
        let mut last = ']';
        for new in chars {
            count += 1;
            if new == '\x07' || (new == '\\' && last == CSI.0) {
                break;
            }
            last = new;
        }
    }

    count
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_normalize_stdout() {
        assert_eq!(normalize_stdout(b"\t"), b"    ");
        // Make sure we don't miss any tabs in edge cases.
        assert_eq!(normalize_stdout(b"\t\t\t\t\t"), b"                    ");
        // Make sure tab is elastic (from 1 space to TAB_SIZE spaces).
        assert_eq!(normalize_stdout(b"\t12345"), b"    12345");
        assert_eq!(normalize_stdout(b"1\t2345"), b"1   2345");
        assert_eq!(normalize_stdout(b"12\t345"), b"12  345");
        assert_eq!(normalize_stdout(b"123\t45"), b"123 45");
        assert_eq!(normalize_stdout(b"1234\t5"), b"1234    5");
        // Make sure we reset alignment on new lines.
        assert_eq!(normalize_stdout(b"123\t\n4\t5"), b"123 \n4   5");
        assert_eq!(normalize_stdout(b"12\t3\n4\t5"), b"12  3\n4   5");
        assert_eq!(normalize_stdout(b"1\t23\n4\t5"), b"1   23\n4   5");
        assert_eq!(normalize_stdout(b"\t123\n4\t5"), b"    123\n4   5");
        assert_eq!(
            normalize_stdout("あ\tい\nう\tえ".as_bytes()),
            "あ  い\nう  え".as_bytes()
        );
        assert_eq!(
            normalize_stdout(b"\x1b[34ma\t\x1b[39mb\x1b[0m"),
            b"\x1b[34ma   \x1b[39mb\x1b[0m"
        );
    }

    #[test]
    fn test_normalize_stdout_uses_grapheme_width_for_tabs() {
        assert_eq!(
            normalize_stdout("e\u{301}\tX".as_bytes()),
            "e\u{301}   X".as_bytes()
        );
    }

    #[test]
    fn test_normalize_stdout_preserves_sgr_while_expanding_tabs() {
        assert_eq!(
            normalize_stdout("\x1b[31me\u{301}\t\x1b[0mX".as_bytes()),
            "\x1b[31me\u{301}   \x1b[0mX".as_bytes()
        );
    }

    #[test]
    fn test_normalize_stdout_preserves_osc_with_grapheme_payload() {
        assert_eq!(
            normalize_stdout("\x1b]0;e\u{301}\x07\tX".as_bytes()),
            "\x1b]0;e\u{301}\x07    X".as_bytes()
        );
    }

    #[test]
    fn test_normalize_stdout_handles_ansi_inside_grapheme() {
        assert_eq!(
            normalize_stdout("e\x1b[31m\u{301}\tX".as_bytes()),
            "e\x1b[31m\u{301}   X".as_bytes()
        );
    }

    #[test]
    fn test_normalize_stdout_handles_multiple_ansi_inside_one_grapheme() {
        assert_eq!(
            normalize_stdout("e\x1b[31m\u{301}\x1b[0m\u{323}\tX".as_bytes()),
            "e\x1b[31m\u{301}\x1b[0m\u{323}   X".as_bytes()
        );
    }

    #[test]
    fn test_normalize_stdout_handles_zwj_emoji_split_by_ansi() {
        assert_eq!(
            normalize_stdout("A👩\u{200d}\x1b[32m💻\tX".as_bytes()),
            "A👩\u{200d}\x1b[32m💻 X".as_bytes()
        );
    }

    #[test]
    fn test_normalize_stdout_resets_crlf_split_by_ansi() {
        assert_eq!(
            normalize_stdout(b"12\r\x1b[31m\n\tX"),
            b"12\r\x1b[31m\n    X"
        );
    }

    #[test]
    fn test_normalize_stdout_ignores_osc_payload_width() {
        assert_eq!(
            normalize_stdout("A\x1b]0;e\u{301}\tignored\x07\tX".as_bytes()),
            "A\x1b]0;e\u{301}\tignored\x07   X".as_bytes()
        );
    }

    #[test]
    fn test_normalize_stdout_handles_mixed_sequences() {
        assert_eq!(
            normalize_stdout(
                "あ\x1b[31me\x1b[0m\u{301}\tZ\r\n\x1b]0;t\x07👩\u{200d}\x1b[32m💻\tY".as_bytes()
            ),
            "あ\x1b[31me\x1b[0m\u{301} Z\r\n\x1b]0;t\x07👩\u{200d}\x1b[32m💻  Y".as_bytes()
        );
    }

    #[test]
    fn test_normalize_stdout_does_not_panic_on_trailing_escape() {
        assert_eq!(normalize_stdout(b"\x1b"), b"\x1b");
    }

    #[test]
    fn test_normalize_stdout_resets_width_after_crlf() {
        assert_eq!(normalize_stdout(b"12\r\n\tX"), b"12\r\n    X");
    }

    #[test]
    fn test_normalize_stdout_handles_c0_width_state() {
        assert_eq!(normalize_stdout(b"\n\x1a\tX"), b"\n\x1a    X");
        assert_eq!(normalize_stdout(b"123\x08\tX"), b"123\x08  X");
        assert_eq!(normalize_stdout(b"123\x0b\tX"), b"123\x0b    X");
        assert_eq!(normalize_stdout(b"123\x0c\tX"), b"123\x0c    X");
        assert_eq!(normalize_stdout(b"123\x1a\tX"), b"123\x1a X");
        assert_eq!(normalize_stdout(b"123\x7f\tX"), b"123\x7f X");
        assert_eq!(
            normalize_stdout("e\x1a\u{301}\tX".as_bytes()),
            "e\x1a\u{301}   X".as_bytes()
        );
    }
}
