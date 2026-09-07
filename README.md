# takumi-cli

Renders an html document from stdin to a paged pdf on stdout with [takumi](https://takumi.kane.tw)'s pdf backend.

```bash
takumi-cli --page-size a4 --margin 10mm < invoice.html > invoice.pdf
```

## Install

Prebuilt binaries for linux (amd64) and macOS (Apple Silicon) are attached to each [GitHub release](https://github.com/remotely-works/takumi-cli/releases). Or build from source:

```bash
cargo build --release
./target/release/takumi-cli --help
```

## Flags

| Flag | Default | Notes |
| --- | --- | --- |
| `-o, --output <path>` | stdout | Where the pdf goes. |
| `--page-size <size>` | `a4` | `a3`, `a4`, `a5`, `b4`, `b5`, `jis-b4`, `jis-b5`, `ledger`, `legal`, `letter`, or `<width>x<height>` in css px. |
| `--landscape` | off | Swaps width and height. |
| `--margin <length>` | `10mm` | Every side; `px`, `mm`, `cm`, `pt` or `in`. |
| `--fonts-dir <dir>` | none | Extra fonts (ttf, otf, ttc, woff, woff2), repeatable. Files are registered in path order. |

The document's `<title>` becomes the pdf title and `<html lang>` its language. `<style>` blocks are applied; `<link>` stylesheets, `@import` and scripts are ignored, and `<img>` tags with http(s) sources render blank with a warning: nothing is fetched.

## Fonts

`fonts/InterVariable.ttf` ([Inter](https://rsms.me/inter/) 4.1, [OFL](fonts/OFL.txt)) is embedded and registered as the `Inter` family and the `sans-serif` generic, so `font-family: 'Inter', Arial, Helvetica, sans-serif` and plain `sans-serif` both land on it. Text no registered font covers fails the render instead of drawing tofu; pass `--fonts-dir` with fonts for those scripts (for example Debian's `fonts-noto-core` package, at `/usr/share/fonts/truetype/noto`).

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Rendered. Warnings, if any, on stderr. |
| 1 | Render failed (bad html, undecodable image, io error). |
| 2 | Bad flags, or an unreadable `--fonts-dir`. |
| 3 | A character has no glyph in any registered font; stderr names it. |

## Upgrading takumi

`takumi-core`, `takumi-html` and `takumi-pdf` are pinned to one git revision in `Cargo.toml` (`takumi-pdf` is not published to crates.io). Bump the three `rev`s together, run `cargo update` for the lock, `cargo test`, and check whether `src/pdf_fixups.rs` is still needed.
