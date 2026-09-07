//! What `takumi_html::from_html` drops from a document and this cli still
//! needs: the `<style>` blocks, the `<title>`, the root `lang`, and the
//! images takumi would silently render blank because it never fetches them.

use html5ever::{ParseOpts, parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

#[derive(Debug, Default, PartialEq)]
pub struct Document {
    /// Contents of every `<style>` element, in document order.
    pub css: Vec<String>,
    pub title: Option<String>,
    pub lang: Option<String>,
    /// `<img src>` values that point at http(s) urls.
    pub remote_images: Vec<String>,
}

pub fn scan(source: &str) -> Document {
    let dom = parse_document(RcDom::default(), ParseOpts::default()).one(source);
    let mut document = Document::default();

    walk(&dom.document, &mut document);
    document
}

fn walk(handle: &Handle, document: &mut Document) {
    if let NodeData::Element { name, attrs, .. } = &handle.data {
        let attribute = |wanted: &str| {
            attrs
                .borrow()
                .iter()
                .find(|attr| attr.name.local.as_ref() == wanted)
                .map(|attr| attr.value.to_string())
        };

        match name.local.as_ref() {
            "style" => document.css.push(text(handle)),
            "title" if document.title.is_none() => {
                document.title = Some(text(handle).trim().to_owned()).filter(|t| !t.is_empty());
            }
            "html" if document.lang.is_none() => document.lang = attribute("lang"),
            "img" => {
                if let Some(src) = attribute("src").filter(|src| is_remote(src)) {
                    document.remote_images.push(src);
                }
            }
            _ => {}
        }
    }

    for child in handle.children.borrow().iter() {
        walk(child, document);
    }
}

fn text(handle: &Handle) -> String {
    handle
        .children
        .borrow()
        .iter()
        .filter_map(|child| match &child.data {
            NodeData::Text { contents } => Some(contents.borrow().to_string()),
            _ => None,
        })
        .collect()
}

fn is_remote(src: &str) -> bool {
    let src = src.trim_start();

    src.len() >= 7
        && (src[..7].eq_ignore_ascii_case("http://")
            || src[..8.min(src.len())].eq_ignore_ascii_case("https://"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_style_blocks_in_order() {
        let document = scan(
            r#"<!DOCTYPE html><html><head>
            <style type="text/css">@import url('https://fonts.googleapis.com/css2?family=Inter');</style>
            <link rel="stylesheet" href="https://example.com/x.css">
            <style>body { color: red }</style>
            </head><body><style>p { margin: 0 }</style><p>x</p></body></html>"#,
        );

        assert_eq!(
            document.css,
            vec![
                "@import url('https://fonts.googleapis.com/css2?family=Inter');".to_owned(),
                "body { color: red }".to_owned(),
                "p { margin: 0 }".to_owned(),
            ]
        );
    }

    #[test]
    fn extracts_title_and_lang() {
        let document = scan(
            r#"<html lang="en"><head><title> Invoice - 42 </title></head><body></body></html>"#,
        );

        assert_eq!(document.title.as_deref(), Some("Invoice - 42"));
        assert_eq!(document.lang.as_deref(), Some("en"));
    }

    #[test]
    fn fragment_without_head_has_no_metadata() {
        let document = scan("<div>hi</div>");

        assert_eq!(document, Document::default());
    }

    #[test]
    fn ignores_commented_out_style() {
        let document = scan("<!-- <style>a{}</style> --><p>x</p>");

        assert!(document.css.is_empty());
    }

    #[test]
    fn detects_remote_images_only() {
        let document = scan(
            r#"<img src="data:image/png;base64,iVBORw0KGgo=" alt="">
            <img src="https://example.com/logo.png">
            <img src="HTTP://example.com/other.png">"#,
        );

        assert_eq!(
            document.remote_images,
            vec![
                "https://example.com/logo.png".to_owned(),
                "HTTP://example.com/other.png".to_owned()
            ]
        );
    }
}
