//! Renders html from stdin to a paged pdf on stdout with takumi.

mod error;
mod fonts;
mod html;
mod page;
mod pdf_fixups;
mod render;

use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser;

use crate::error::CliError;

#[derive(Parser)]
#[command(
    name = "takumi-cli",
    version,
    about = "Renders html from stdin to a paged pdf on stdout."
)]
struct Cli {
    /// Write the pdf to this file instead of stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Page size: a3, a4, a5, b4, b5, jis-b4, jis-b5, ledger, legal, letter, or <width>x<height> in css px.
    #[arg(long, default_value = "a4")]
    page_size: String,
    /// Swap the page width and height.
    #[arg(long)]
    landscape: bool,
    /// Margin on every side, as a css length (px, mm, cm, pt, in).
    #[arg(long, default_value = "10mm")]
    margin: String,
    /// Directory of extra fonts (ttf, otf, ttc, woff, woff2) for glyphs the embedded Inter lacks. Repeatable.
    #[arg(long = "fonts-dir")]
    fonts_dirs: Vec<PathBuf>,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("takumi-cli: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    let page = page::page_options(&cli.page_size, cli.landscape, &cli.margin)?;
    let fonts = fonts::load(&cli.fonts_dirs)?;

    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source)?;

    let rendered = render::render(&source, &fonts, page)?;

    for src in &rendered.remote_images {
        eprintln!("takumi-cli: warning: remote image not fetched: {src}");
    }

    match cli.output {
        Some(path) => fs::write(path, rendered.pdf)?,
        None => {
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(&rendered.pdf)?;
            stdout.flush()?;
        }
    }

    Ok(())
}
