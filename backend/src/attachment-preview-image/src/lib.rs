#![forbid(unsafe_code)]

use std::io::Cursor;

use hermes_attachment_preview_api::{
    ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1,
    wire::{AttachmentPreviewContentTypeV1, AttachmentPreviewKindV1},
};
use hermes_attachment_preview_renderer_contract::{
    ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1, AttachmentPreviewRenderRequestV1,
    AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1, AttachmentPreviewRendererV1,
    AttachmentPreviewSourceFormatV1,
};
use image::{GenericImageView, ImageFormat, ImageReader};

pub const PACKAGE: &str = "hermes-attachment-preview-image";
const MAX_IMAGE_DIMENSION_V1: u32 = 16_384;

#[derive(Clone, Copy, Debug, Default)]
pub struct AttachmentPreviewImageRendererV1;

impl AttachmentPreviewRendererV1 for AttachmentPreviewImageRendererV1 {
    fn render(
        &self,
        request: AttachmentPreviewRenderRequestV1<'_>,
    ) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1> {
        if !matches!(
            request.source_format,
            AttachmentPreviewSourceFormatV1::Png
                | AttachmentPreviewSourceFormatV1::Jpeg
                | AttachmentPreviewSourceFormatV1::Gif
                | AttachmentPreviewSourceFormatV1::Webp
        ) {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        let reader = ImageReader::new(Cursor::new(request.source_bytes))
            .with_guessed_format()
            .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
        let image = reader
            .decode()
            .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
        let (width, height) = image.dimensions();
        let pixels = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or(AttachmentPreviewRendererErrorV1::InvalidContent)?;
        if width == 0
            || height == 0
            || width > MAX_IMAGE_DIMENSION_V1
            || height > MAX_IMAGE_DIMENSION_V1
            || pixels > ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1
        {
            return Err(AttachmentPreviewRendererErrorV1::SourceTooLarge);
        }
        let mut output = Cursor::new(Vec::new());
        image
            .write_to(&mut output, ImageFormat::Png)
            .map_err(|_| AttachmentPreviewRendererErrorV1::Failed)?;
        let bytes = output.into_inner();
        if bytes.is_empty() || bytes.len() as u64 > ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1 {
            return Err(AttachmentPreviewRendererErrorV1::OutputTooLarge);
        }
        Ok(AttachmentPreviewRenderResultV1 {
            preview_kind: AttachmentPreviewKindV1::Image,
            content_type: AttachmentPreviewContentTypeV1::Png,
            bytes,
            truncated: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgba, RgbaImage};

    fn source_png() -> Vec<u8> {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 1, Rgba([1, 2, 3, 255])));
        let mut bytes = Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    #[test]
    fn decodes_and_reencodes_to_a_fresh_png() {
        let source = source_png();
        let result = AttachmentPreviewImageRendererV1
            .render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Png,
                source_bytes: &source,
            })
            .unwrap();
        assert!(result.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(result.content_type, AttachmentPreviewContentTypeV1::Png);
        assert!(!result.truncated);
    }

    #[test]
    fn malformed_and_wrong_format_fail_closed() {
        assert_eq!(
            AttachmentPreviewImageRendererV1.render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Png,
                source_bytes: b"\x89PNG\r\n\x1a\n",
            }),
            Err(AttachmentPreviewRendererErrorV1::InvalidContent)
        );
        assert_eq!(
            AttachmentPreviewImageRendererV1.render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Pdf,
                source_bytes: b"%PDF-1.7",
            }),
            Err(AttachmentPreviewRendererErrorV1::Unsupported)
        );
    }
}
