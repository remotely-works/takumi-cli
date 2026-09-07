use std::{
    fs,
    path::{Path, PathBuf},
};

use takumi_core::resources::font::{FontOverride, FontResource, FontSource, Fonts};

use crate::error::CliError;

/// Inter is the default sans-serif for rendered documents. The variable font
/// carries every weight, so one file covers regular through bold.
static INTER: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

const FONT_EXTENSIONS: [&str; 5] = ["ttf", "otf", "ttc", "woff", "woff2"];

/// The embedded Inter, serving both the `Inter` family and the `sans-serif`
/// generic (so `Arial, Helvetica, sans-serif` fallbacks land on it too), plus
/// every font file found in `dirs` for the scripts Inter does not cover.
pub fn load(dirs: &[PathBuf]) -> Result<Fonts, CliError> {
    let mut fonts = Fonts::default();

    fonts.register(
        FontResource::new(FontSource::from_static(INTER))
            .override_info(FontOverride {
                family_name: Some("Inter".into()),
                ..Default::default()
            })
            .generic_family("sans-serif".parse()?),
    )?;

    for dir in dirs {
        for path in font_files(dir)? {
            fonts.register(FontResource::new(fs::read(&path)?))?;
        }
    }

    Ok(fonts)
}

/// Sorted so the fallback order between fonts of equal rank is stable.
fn font_files(dir: &Path) -> Result<Vec<PathBuf>, CliError> {
    let entries = fs::read_dir(dir).map_err(|error| {
        CliError::Usage(format!(
            "could not read --fonts-dir {}: {error}",
            dir.display()
        ))
    })?;
    let mut files: Vec<_> = entries
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    FONT_EXTENSIONS
                        .iter()
                        .any(|known| known.eq_ignore_ascii_case(extension))
                })
        })
        .collect();

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_inter_registers() {
        load(&[]).unwrap();
    }

    #[test]
    fn missing_fonts_dir_is_a_usage_error() {
        let error = load(&[PathBuf::from("/definitely/not/here")])
            .err()
            .expect("expected an error");

        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn font_files_are_filtered_and_sorted() {
        let dir = std::env::temp_dir().join(format!("takumi-cli-fonts-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        for name in ["b.TTF", "a.woff2", "readme.txt", "c.otf"] {
            fs::write(dir.join(name), b"").unwrap();
        }

        let files = font_files(&dir).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(
            files
                .iter()
                .map(|path| path.file_name().unwrap().to_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["a.woff2", "b.TTF", "c.otf"]
        );
    }
}
