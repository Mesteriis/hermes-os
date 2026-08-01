use hermes_attachment_text_extraction_core::{
    AttachmentTextFormatV1, normalize_attachment_text_v1,
};
use hermes_attachment_text_extraction_docx::extract_docx_text_v1;
use hermes_attachment_text_extraction_ocr::{TesseractOcrConfigurationV1, extract_image_text_v1};
use hermes_attachment_text_extraction_parser_contract::{
    AttachmentTextParserErrorV1, AttachmentTextParserKindV1, detect_attachment_text_parser_v1,
};
use hermes_attachment_text_extraction_pdf::extract_pdf_text_v1;
use hermes_attachment_text_extraction_plain::extract_plain_text_v1;
use sha2::{Digest, Sha256};

pub struct AttachmentTextExtractionParserRuntimeV1 {
    ocr: Option<TesseractOcrConfigurationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTextRuntimeParseResultV1 {
    pub text_utf8: Vec<u8>,
    pub format: AttachmentTextFormatV1,
    pub extraction_truncated: bool,
    pub parser_identity_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextRuntimeParseErrorV1 {
    Unsupported,
    SourceTooLarge,
    InvalidContent,
    ParserUnavailable,
    ParserFailed,
}

impl AttachmentTextExtractionParserRuntimeV1 {
    #[must_use]
    pub const fn new(ocr: Option<TesseractOcrConfigurationV1>) -> Self {
        Self { ocr }
    }

    pub fn extract(
        &self,
        source: &[u8],
    ) -> Result<AttachmentTextRuntimeParseResultV1, AttachmentTextRuntimeParseErrorV1> {
        let parser = detect_attachment_text_parser_v1(source).map_err(map_error)?;
        let output = match parser {
            AttachmentTextParserKindV1::PlainUtf8 => extract_plain_text_v1(source),
            AttachmentTextParserKindV1::Pdf => extract_pdf_text_v1(source),
            AttachmentTextParserKindV1::Docx => extract_docx_text_v1(source),
            AttachmentTextParserKindV1::Ocr => self
                .ocr
                .as_ref()
                .ok_or(AttachmentTextParserErrorV1::ParserUnavailable)
                .and_then(|configuration| extract_image_text_v1(source, configuration)),
        }
        .map_err(map_error)?;
        let normalized =
            normalize_attachment_text_v1(&output.text_utf8, output.extraction_truncated)
                .map_err(|_| AttachmentTextRuntimeParseErrorV1::InvalidContent)?;
        Ok(AttachmentTextRuntimeParseResultV1 {
            text_utf8: normalized.bytes,
            format: format(parser),
            extraction_truncated: normalized.extraction_truncated,
            parser_identity_sha256: parser_identity(parser),
        })
    }
}

const fn format(parser: AttachmentTextParserKindV1) -> AttachmentTextFormatV1 {
    match parser {
        AttachmentTextParserKindV1::PlainUtf8 => AttachmentTextFormatV1::PlainUtf8,
        AttachmentTextParserKindV1::Pdf => AttachmentTextFormatV1::Pdf,
        AttachmentTextParserKindV1::Docx => AttachmentTextFormatV1::Docx,
        AttachmentTextParserKindV1::Ocr => AttachmentTextFormatV1::Ocr,
    }
}

fn parser_identity(parser: AttachmentTextParserKindV1) -> [u8; 32] {
    let label = match parser {
        AttachmentTextParserKindV1::PlainUtf8 => b"plain-v1".as_slice(),
        AttachmentTextParserKindV1::Pdf => b"pdf-text-extract-0.2.0-v1".as_slice(),
        AttachmentTextParserKindV1::Docx => b"docx-quick-xml-0.41.0-v1".as_slice(),
        AttachmentTextParserKindV1::Ocr => b"tesseract-eng-rus-v1".as_slice(),
    };
    let mut digest = Sha256::new();
    digest.update(b"hermes.attachment-text-extraction.parser-identity.v1\0");
    digest.update(label);
    digest.finalize().into()
}

const fn map_error(error: AttachmentTextParserErrorV1) -> AttachmentTextRuntimeParseErrorV1 {
    match error {
        AttachmentTextParserErrorV1::SourceTooLarge => {
            AttachmentTextRuntimeParseErrorV1::SourceTooLarge
        }
        AttachmentTextParserErrorV1::UnsupportedFormat => {
            AttachmentTextRuntimeParseErrorV1::Unsupported
        }
        AttachmentTextParserErrorV1::ParserUnavailable
        | AttachmentTextParserErrorV1::ParserTimedOut => {
            AttachmentTextRuntimeParseErrorV1::ParserUnavailable
        }
        AttachmentTextParserErrorV1::ParserFailed => {
            AttachmentTextRuntimeParseErrorV1::ParserFailed
        }
        AttachmentTextParserErrorV1::EmptySource
        | AttachmentTextParserErrorV1::InvalidContent
        | AttachmentTextParserErrorV1::EncryptedContent
        | AttachmentTextParserErrorV1::OutputTooLarge => {
            AttachmentTextRuntimeParseErrorV1::InvalidContent
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_plain_bytes_to_the_exact_adapter_and_normalizes_output() {
        let runtime = AttachmentTextExtractionParserRuntimeV1::new(None);
        let result = runtime
            .extract(b"first\r\nsecond")
            .expect("plain extraction");
        assert_eq!(result.format, AttachmentTextFormatV1::PlainUtf8);
        assert_eq!(result.text_utf8, b"first\nsecond");
        assert!(result.parser_identity_sha256.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn image_without_verified_ocr_configuration_fails_closed() {
        let runtime = AttachmentTextExtractionParserRuntimeV1::new(None);
        assert_eq!(
            runtime.extract(b"\x89PNG\r\n\x1a\nbody"),
            Err(AttachmentTextRuntimeParseErrorV1::ParserUnavailable)
        );
    }
}
