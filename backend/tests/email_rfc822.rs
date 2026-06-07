use hermes_hub_backend::domains::mail::rfc822::{
    ParsedEmailAttachmentDisposition, parse_rfc822_message,
};

#[test]
fn rfc822_parser_extracts_nested_multipart_attachments_for_current_basic_slice() {
    let raw = concat!(
        "Subject: Nested attachments\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "Content-Type: multipart/mixed; boundary=\"outer-boundary\"\r\n",
        "\r\n",
        "--outer-boundary\r\n",
        "Content-Type: multipart/alternative; boundary=\"alt-boundary\"\r\n",
        "\r\n",
        "--alt-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "Nested=20plain=20body.\r\n",
        "--alt-boundary\r\n",
        "Content-Type: text/html; charset=utf-8\r\n",
        "\r\n",
        "<p>Nested HTML body.</p>\r\n",
        "--alt-boundary--\r\n",
        "--outer-boundary\r\n",
        "Content-Type: application/pdf; name*=UTF-8''report%20Q2.pdf\r\n",
        "Content-Disposition: attachment; filename*=UTF-8''report%20Q2.pdf\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQ=\r\n",
        "--outer-boundary\r\n",
        "Content-Type: text/plain; name=\"notes.txt\"\r\n",
        "Content-Disposition: inline; filename=\"notes.txt\"\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "note=20body=0Asecond=20line\r\n",
        "--outer-boundary--\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse nested multipart message");

    assert_eq!(parsed.subject, "Nested attachments");
    assert_eq!(parsed.body_text, "Nested plain body.");
    assert_eq!(parsed.attachments.len(), 2);

    let pdf = &parsed.attachments[0];
    assert_eq!(pdf.provider_attachment_id, "part-1");
    assert_eq!(pdf.filename.as_deref(), Some("report Q2.pdf"));
    assert_eq!(pdf.content_type, "application/pdf");
    assert_eq!(
        pdf.disposition,
        ParsedEmailAttachmentDisposition::Attachment
    );
    assert_eq!(pdf.body_bytes, b"%PDF-1.4");

    let notes = &parsed.attachments[1];
    assert_eq!(notes.provider_attachment_id, "part-2");
    assert_eq!(notes.filename.as_deref(), Some("notes.txt"));
    assert_eq!(notes.content_type, "text/plain");
    assert_eq!(notes.disposition, ParsedEmailAttachmentDisposition::Inline);
    assert_eq!(notes.body_bytes, b"note body\nsecond line");
}

#[test]
fn rfc822_parser_extracts_rfc2231_continued_attachment_filenames() {
    let raw = concat!(
        "Subject: Continued filename\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "Content-Type: multipart/mixed; boundary=\"continued-boundary\"\r\n",
        "\r\n",
        "--continued-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Body.\r\n",
        "--continued-boundary\r\n",
        "Content-Type: application/pdf;\r\n",
        " name*0*=UTF-8''quarterly%20;\r\n",
        " name*1*=%D1%84%D0%B0%D0%B9%D0%BB;\r\n",
        " name*2=.pdf\r\n",
        "Content-Disposition: attachment;\r\n",
        " filename*0*=UTF-8''quarterly%20;\r\n",
        " filename*1*=%D1%84%D0%B0%D0%B9%D0%BB;\r\n",
        " filename*2=.pdf\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQ=\r\n",
        "--continued-boundary--\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse continued filename message");

    assert_eq!(parsed.attachments.len(), 1);
    let attachment = &parsed.attachments[0];
    assert_eq!(attachment.filename.as_deref(), Some("quarterly файл.pdf"));
    assert_eq!(attachment.content_type, "application/pdf");
    assert_eq!(attachment.body_bytes, b"%PDF-1.4");
}
