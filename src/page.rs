use takumi_core::units::{ONE_CM_IN_PX, ONE_IN_PX, ONE_MM_IN_PX, ONE_PT_IN_PX};
use takumi_pdf::{PageMargins, PageOptions};

use crate::error::CliError;

/// Builds the page geometry from the cli flags: a css paged media size
/// keyword or `<width>x<height>` in css px, and a uniform margin length.
pub fn page_options(size: &str, landscape: bool, margin: &str) -> Result<PageOptions, CliError> {
    let mut page = page_size(size)?;

    if landscape {
        page = page.landscape();
    }

    page.margin = PageMargins::uniform(length_px(margin).map_err(|_| {
        CliError::Usage(format!(
            "invalid --margin {margin:?}: expected a length such as 10mm, 13.3px or 0.5in"
        ))
    })?);

    Ok(page)
}

fn page_size(size: &str) -> Result<PageOptions, CliError> {
    let preset = match size.to_ascii_lowercase().as_str() {
        "a3" => Some(PageOptions::A3),
        "a4" => Some(PageOptions::A4),
        "a5" => Some(PageOptions::A5),
        "b4" => Some(PageOptions::B4),
        "b5" => Some(PageOptions::B5),
        "jis-b4" => Some(PageOptions::JIS_B4),
        "jis-b5" => Some(PageOptions::JIS_B5),
        "ledger" => Some(PageOptions::LEDGER),
        "legal" => Some(PageOptions::LEGAL),
        "letter" => Some(PageOptions::LETTER),
        _ => None,
    };

    if let Some(preset) = preset {
        return Ok(preset);
    }

    let invalid = || {
        CliError::Usage(format!(
            "invalid --page-size {size:?}: expected a3, a4, a5, b4, b5, jis-b4, jis-b5, ledger, legal, letter or <width>x<height> in px"
        ))
    };
    let (width, height) = size.split_once('x').ok_or_else(invalid)?;
    let width = length_px(width).map_err(|_| invalid())?;
    let height = length_px(height).map_err(|_| invalid())?;

    if width <= 0.0 || height <= 0.0 {
        return Err(invalid());
    }

    Ok(PageOptions {
        width,
        height,
        margin: PageMargins::AUTO,
    })
}

/// Parses a css absolute length into px. A bare number is px.
fn length_px(input: &str) -> Result<f32, ()> {
    let input = input.trim();
    let (number, factor) = [
        ("px", 1.0),
        ("mm", ONE_MM_IN_PX),
        ("cm", ONE_CM_IN_PX),
        ("pt", ONE_PT_IN_PX),
        ("in", ONE_IN_PX),
    ]
    .into_iter()
    .find_map(|(unit, factor)| input.strip_suffix(unit).map(|number| (number, factor)))
    .unwrap_or((input, 1.0));
    let value: f32 = number.trim().parse().map_err(|_| ())?;

    if !value.is_finite() || value < 0.0 {
        return Err(());
    }

    Ok(value * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn margin(page: &PageOptions) -> f32 {
        match page.margin.top {
            takumi_pdf::PageMargin::Px(value) => value,
            takumi_pdf::PageMargin::Auto => panic!("expected a px margin"),
        }
    }

    #[test]
    fn a4_portrait_with_default_margin() {
        let page = page_options("a4", false, "10mm").unwrap();

        assert!((page.width - 793.7).abs() < 0.1, "{}", page.width);
        assert!((page.height - 1122.5).abs() < 0.1, "{}", page.height);
        assert!((margin(&page) - 37.795).abs() < 0.01);
    }

    #[test]
    fn letter_landscape_swaps_dimensions() {
        let portrait = page_options("letter", false, "0").unwrap();
        let landscape = page_options("Letter", true, "0").unwrap();

        assert_eq!(
            (landscape.width, landscape.height),
            (portrait.height, portrait.width)
        );
        assert_eq!(portrait.width, 8.5 * 96.0);
    }

    #[test]
    fn custom_size_in_px() {
        let page = page_options("800x600", false, "10pt").unwrap();

        assert_eq!((page.width, page.height), (800.0, 600.0));
        assert!((margin(&page) - 13.333).abs() < 0.01);
    }

    #[test]
    fn margin_units() {
        assert_eq!(margin(&page_options("a4", false, "12").unwrap()), 12.0);
        assert_eq!(margin(&page_options("a4", false, "12px").unwrap()), 12.0);
        assert_eq!(margin(&page_options("a4", false, "0.5in").unwrap()), 48.0);
    }

    #[test]
    fn rejects_bad_input() {
        for (size, margin) in [
            ("a7", "10mm"),
            ("800", "10mm"),
            ("0x600", "10mm"),
            ("a4", "ten"),
            ("a4", "-1mm"),
            ("a4", "10em"),
        ] {
            let error = page_options(size, false, margin)
                .err()
                .expect("expected an error");

            assert_eq!(error.exit_code(), 2, "{size} {margin}");
        }
    }
}
