pub(super) fn message_html_references_url(body_html: &str, image_url: &str) -> bool {
    body_html.contains(image_url)
        || body_html.contains(&image_url.replace('&', "&amp;"))
        || body_html.replace("&amp;", "&").contains(image_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_escaped_message_image_references() {
        let html = r#"<img src="https://img.example.test/a.png?x=1&amp;y=2">"#;
        assert!(message_html_references_url(
            html,
            "https://img.example.test/a.png?x=1&y=2"
        ));
        assert!(!message_html_references_url(
            html,
            "https://img.example.test/other.png"
        ));
    }
}
