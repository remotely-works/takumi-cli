//! Repairs for output takumi-pdf writes that viewers reject.

/// `/DW` (the default glyph width of a CID font) is an integer in the PDF
/// spec (ISO 32000-1 table 117). takumi-pdf 0.14 writes it as a real when the
/// most common glyph width is fractional, and poppler and Ghostscript then
/// ignore it and fall back to 1000, so every glyph left out of `/W` renders
/// with a gap after it. Rounding in place, padded with spaces, keeps every
/// byte offset in the file valid. Drop this once upstream rounds `/DW`.
pub fn round_default_widths(pdf: &mut [u8]) {
    const KEY: &[u8] = b"/DW ";
    let mut from = 0;

    while let Some(at) = find(&pdf[from..], KEY) {
        let start = from + at + KEY.len();
        let end = start
            + pdf[start..]
                .iter()
                .take_while(|byte| byte.is_ascii_digit() || **byte == b'.')
                .count();
        from = end;

        let Some(dot) = pdf[start..end].iter().position(|byte| *byte == b'.') else {
            continue;
        };
        let value: f64 = match std::str::from_utf8(&pdf[start..end])
            .ok()
            .and_then(|text| text.parse().ok())
        {
            Some(value) => value,
            None => continue,
        };
        let mut rounded = format!("{}", value.round() as u64);

        if rounded.len() > dot {
            // Rounding up gained a digit (e.g. 999.7): keep the integer part,
            // an error under one thousandth of an em.
            rounded = String::from_utf8_lossy(&pdf[start..start + dot]).into_owned();
        }

        pdf[start..end].fill(b' ');
        pdf[start..start + rounded.len()].copy_from_slice(rounded.as_bytes());
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(input: &str) -> String {
        let mut bytes = input.as_bytes().to_vec();
        round_default_widths(&mut bytes);
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn rounds_fractional_default_widths_keeping_the_length() {
        let input = "<</DW 591.3086/CIDToGIDMap/Identity>> <</DW 634.7656/W[0 0 600]>>";
        let output = fixed(input);

        assert_eq!(
            output,
            "<</DW 591     /CIDToGIDMap/Identity>> <</DW 635     /W[0 0 600]>>"
        );
        assert_eq!(output.len(), input.len());
    }

    #[test]
    fn leaves_integers_and_unrelated_keys_alone() {
        for input in [
            "<</DW 1000/W[]>>",
            "<</DWX 1.5>>",
            "(text /DW 1.5 in a string)",
        ] {
            assert_eq!(
                fixed(input),
                input.replace("/DW 1.5 in", "/DW 2   in"),
                "{input}"
            );
        }
    }

    #[test]
    fn keeps_the_integer_part_when_rounding_would_gain_a_digit() {
        assert_eq!(fixed("/DW 999.7/"), "/DW 999  /");
    }
}
