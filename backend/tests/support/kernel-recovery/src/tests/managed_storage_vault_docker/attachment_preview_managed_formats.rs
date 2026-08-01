//! Managed multi-format Preview evidence over the exact Gateway and Blob boundary.

use std::io::{Cursor, Write};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hermes_attachment_preview_api::{
    ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1, ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
    wire::{
        AttachmentPreviewContentTypeV1, AttachmentPreviewErrorCodeV1, AttachmentPreviewKindV1,
        AttachmentPreviewStateV1, IssueAttachmentPreviewReadRequestV1,
        IssueAttachmentPreviewReadResponseV1, StartAttachmentPreviewRequestV1,
        StartAttachmentPreviewResponseV1,
    },
};
use hyper::StatusCode;
use prost::Message;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use super::*;
use super::{
    attachment_preview_gateway_fixture::{
        AttachmentPreviewGateway, post_attachment_preview_proto_v1,
        read_attachment_preview_blob_v1, read_terminal_attachment_preview_sse_event_v1,
        wait_for_ready_attachment_preview_v1,
    },
    attachment_security_blob_fixture::AttachmentSecurityBlobSourceFixture,
    attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture,
    attachment_security_event_flow::{
        assert_clean_attachment_security_verdict_flow, prepare_communications_attachment_for_scan,
    },
    mail_attachment_flow::wait_for_attachment_state,
};

#[derive(Clone, Copy)]
enum ManagedPreviewOutputV1 {
    FreshPng,
    ExactSource,
}

struct ManagedPreviewFormatV1 {
    label: &'static str,
    source: Vec<u8>,
    preview_kind: AttachmentPreviewKindV1,
    content_type: AttachmentPreviewContentTypeV1,
    truncated: bool,
    output: ManagedPreviewOutputV1,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn assert_managed_attachment_preview_formats_v1(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    data: &Path,
    blob_source: &AttachmentSecurityBlobSourceFixture,
    clamav: &AttachmentSecurityClamAvFixture,
    router: &AttachmentPreviewGateway,
    gateway_runtime: &tokio::runtime::Runtime,
    cookie: &str,
) {
    let filter = std::env::var("HERMES_ATTACHMENT_PREVIEW_MANAGED_FORMAT_FILTER").ok();
    let formats = managed_preview_formats_v1();
    if let Some(filter) = filter.as_deref() {
        assert!(
            formats.iter().any(|format| format.label == filter),
            "unknown managed Preview format filter"
        );
    }
    for (index, format) in formats
        .into_iter()
        .filter(|format| {
            filter
                .as_deref()
                .is_none_or(|filter| format.label == filter)
        })
        .enumerate()
    {
        let discriminator = u8::try_from(index).expect("bounded Preview format index");
        let blob = blob_source.write(
            store,
            supervisor,
            data,
            [0xb0_u8.checked_add(discriminator).expect("Preview Blob id"); 16],
            &format.source,
        );
        let attachment = prepare_communications_attachment_for_scan(
            store,
            format.label,
            blob.declared_size,
            blob.receipt_sha256,
        );
        assert_clean_attachment_security_verdict_flow(
            store,
            &attachment,
            &blob,
            clamav,
            &format.source,
        );
        assert_eq!(
            wait_for_attachment_state(store, supervisor, attachment.attachment_anchor_id),
            hermes_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
                as u32
        );

        let accepted = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
            router,
            gateway_runtime,
            cookie,
            ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
            StartAttachmentPreviewRequestV1 {
                protocol_major: 1,
                operation_id: vec![
                    0xc0_u8
                        .checked_add(discriminator)
                        .expect("Preview operation id");
                    16
                ],
                attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
            },
        );
        assert_eq!(
            accepted.error,
            AttachmentPreviewErrorCodeV1::Unspecified as i32,
            "{} Preview request",
            format.label
        );
        let ready = wait_for_ready_attachment_preview_v1(
            router,
            gateway_runtime,
            cookie,
            &accepted.run_id,
            format.label,
        );
        assert_eq!(ready.state, AttachmentPreviewStateV1::Ready as i32);
        assert_eq!(ready.preview_kind, format.preview_kind as i32);
        assert_eq!(ready.content_type, format.content_type as i32);
        assert_eq!(ready.truncated, format.truncated);
        assert_private_source_absent_v1(&ready.encode_to_vec(), &format.source);

        let event = read_terminal_attachment_preview_sse_event_v1(
            router,
            gateway_runtime,
            cookie,
            &accepted.run_id,
        );
        assert_private_source_absent_v1(&event.encode_to_vec(), &format.source);

        let ticket = post_attachment_preview_proto_v1::<_, IssueAttachmentPreviewReadResponseV1>(
            router,
            gateway_runtime,
            cookie,
            ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
            IssueAttachmentPreviewReadRequestV1 {
                protocol_major: 1,
                run_id: accepted.run_id,
            },
        );
        assert_eq!(
            ticket.error,
            AttachmentPreviewErrorCodeV1::Unspecified as i32
        );
        let (status, body) = read_attachment_preview_blob_v1(
            router,
            gateway_runtime,
            Some(cookie),
            ticket.opaque_read_ticket.clone(),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ready.preview_size_bytes, body.len() as u64);
        match format.output {
            ManagedPreviewOutputV1::FreshPng => {
                assert!(body.starts_with(b"\x89PNG\r\n\x1a\n"));
                assert_ne!(body, format.source, "{} must be re-rendered", format.label);
            }
            ManagedPreviewOutputV1::ExactSource => assert_eq!(body, format.source),
        }
        assert_eq!(
            read_attachment_preview_blob_v1(
                router,
                gateway_runtime,
                Some(cookie),
                ticket.opaque_read_ticket,
            )
            .0,
            StatusCode::NOT_FOUND,
            "{} read ticket must be one-use",
            format.label
        );
    }
}

fn assert_private_source_absent_v1(carrier: &[u8], source: &[u8]) {
    assert!(
        !carrier
            .windows(source.len())
            .any(|candidate| candidate == source),
        "private source bytes escaped into a metadata-only carrier"
    );
}

fn managed_preview_formats_v1() -> Vec<ManagedPreviewFormatV1> {
    vec![
        ManagedPreviewFormatV1 {
            label: "attachment-preview-image-managed",
            source: STANDARD
                .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
                .expect("decode PNG fixture"),
            preview_kind: AttachmentPreviewKindV1::Image,
            content_type: AttachmentPreviewContentTypeV1::Png,
            truncated: false,
            output: ManagedPreviewOutputV1::FreshPng,
        },
        ManagedPreviewFormatV1 {
            label: "attachment-preview-pdf-managed",
            source: minimal_pdf_v1(),
            preview_kind: AttachmentPreviewKindV1::Document,
            content_type: AttachmentPreviewContentTypeV1::Png,
            truncated: true,
            output: ManagedPreviewOutputV1::FreshPng,
        },
        ManagedPreviewFormatV1 {
            label: "attachment-preview-docx-managed",
            source: minimal_docx_v1(),
            preview_kind: AttachmentPreviewKindV1::Document,
            content_type: AttachmentPreviewContentTypeV1::Png,
            truncated: false,
            output: ManagedPreviewOutputV1::FreshPng,
        },
        ManagedPreviewFormatV1 {
            label: "attachment-preview-mp3-managed",
            source: vec![0xff, 0xfb, 0x90, 0x64],
            preview_kind: AttachmentPreviewKindV1::Audio,
            content_type: AttachmentPreviewContentTypeV1::MpegAudio,
            truncated: false,
            output: ManagedPreviewOutputV1::ExactSource,
        },
        ManagedPreviewFormatV1 {
            label: "attachment-preview-mp4-managed",
            source: minimal_mp4_v1(),
            preview_kind: AttachmentPreviewKindV1::Video,
            content_type: AttachmentPreviewContentTypeV1::Mp4Video,
            truncated: false,
            output: ManagedPreviewOutputV1::ExactSource,
        },
    ]
}

fn minimal_pdf_v1() -> Vec<u8> {
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for object in [
        b"<< /Type /Catalog /Pages 2 0 R >>".as_slice(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".as_slice(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 72 72] /Resources <<>> /Contents 4 0 R >>"
            .as_slice(),
        b"<< /Length 0 >>\nstream\n\nendstream".as_slice(),
    ] {
        offsets.push(bytes.len());
        let number = offsets.len();
        bytes.extend_from_slice(format!("{number} 0 obj\n").as_bytes());
        bytes.extend_from_slice(object);
        bytes.extend_from_slice(b"\nendobj\n");
    }
    let xref = bytes.len();
    bytes.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    bytes
}

fn minimal_docx_v1() -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut bytes));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, content) in [
            (
                "[Content_Types].xml",
                br#"<Types><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.as_slice(),
            ),
            (
                "_rels/.rels",
                br#"<Relationships><Relationship Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.as_slice(),
            ),
            (
                "word/document.xml",
                br#"<w:document xmlns:w="urn:w"><w:body><w:p><w:r><w:t>Managed clean-room DOCX preview</w:t></w:r></w:p></w:body></w:document>"#.as_slice(),
            ),
        ] {
            writer.start_file(name, options).expect("DOCX fixture entry");
            writer.write_all(content).expect("DOCX fixture content");
        }
        writer.finish().expect("finish DOCX fixture");
    }
    bytes
}

fn minimal_mp4_v1() -> Vec<u8> {
    [
        mp4_box_v1(b"ftyp", b"isom\0\0\0\0isom"),
        mp4_box_v1(b"moov", b""),
        mp4_box_v1(b"mdat", b"frame"),
    ]
    .concat()
}

fn mp4_box_v1(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = u32::try_from(8 + payload.len())
        .expect("bounded MP4 fixture box")
        .to_be_bytes()
        .to_vec();
    bytes.extend_from_slice(kind);
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn managed_format_fixtures_cross_the_exact_runtime_renderer_dispatch() {
    let renderer = hermes_attachment_preview_runtime::renderer::AttachmentPreviewRendererRuntimeV1;
    for format in managed_preview_formats_v1() {
        let rendered = renderer
            .render(&format.source)
            .unwrap_or_else(|error| panic!("{} fixture: {error:?}", format.label));
        assert_eq!(rendered.preview_kind, format.preview_kind);
        assert_eq!(rendered.content_type, format.content_type);
        assert_eq!(rendered.truncated, format.truncated);
    }
}
