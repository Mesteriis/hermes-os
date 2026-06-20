#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCommunicationSourceMessage {
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub headers: Vec<(String, String)>,
    pub body_text: String,
    pub body_html: Option<String>,
    pub attachments: Vec<ParsedEmailAttachment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedEmailAttachment {
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub disposition: ParsedEmailAttachmentDisposition,
    pub body_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParsedEmailAttachmentDisposition {
    Attachment,
    Inline,
    Unknown,
}
