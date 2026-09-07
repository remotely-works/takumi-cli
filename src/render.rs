use std::sync::Arc;

use takumi_core::{
    resources::font::Fonts,
    style::{Lang, StyleSheet},
};
use takumi_html::{FromHtmlOptions, from_html};
use takumi_pdf::{PageOptions, PdfMetadata, PdfOptions};

use crate::{error::CliError, html, pdf_fixups};

const UA_CSS: &str = include_str!("ua.css");

pub struct Rendered {
    pub pdf: Vec<u8>,
    /// Image urls the document expects fetched, which takumi renders blank.
    pub remote_images: Vec<String>,
}

pub fn render(source: &str, fonts: &Fonts, page: PageOptions) -> Result<Rendered, CliError> {
    let document = html::scan(source);
    let node = from_html(source, FromHtmlOptions::default())?;
    let stylesheet = Arc::new(StyleSheet::parse_owned_list_loosy(
        std::iter::once(UA_CSS.to_owned())
            .chain(document.css)
            .collect(),
    ));
    let lang = document.lang.as_deref().map(Lang::parse).transpose()?;

    let mut pdf = takumi_pdf::render(
        PdfOptions::builder()
            .node(node)
            .fonts(fonts)
            .stylesheet(stylesheet)
            .page(page)
            .lang(lang)
            .metadata(PdfMetadata {
                title: document.title,
                ..Default::default()
            })
            .build(),
    )?;
    pdf_fixups::round_default_widths(&mut pdf);

    Ok(Rendered {
        pdf,
        remote_images: document.remote_images,
    })
}
