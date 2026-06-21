use super::models::AttachmentCategory;

pub(super) fn classify_by_name_and_type(filename: &str, content_type: &str) -> AttachmentCategory {
    let lower = filename.to_lowercase();

    if lower.contains("invoice") || lower.contains("factura") || lower.contains("receipt") {
        return AttachmentCategory::Invoice;
    }
    if lower.contains("contract") || lower.contains("agreement") || lower.contains("nda") {
        return AttachmentCategory::Contract;
    }
    if lower.contains("certificate") || lower.contains("cert") {
        return AttachmentCategory::Certificate;
    }
    if lower.contains("tax") || lower.contains("hacienda") || lower.contains("aeat") {
        return AttachmentCategory::TaxDocument;
    }
    if lower.contains("passport") || lower.contains("dni") || lower.contains("nie") {
        return AttachmentCategory::IdentityDocument;
    }
    if lower.contains("report") {
        return AttachmentCategory::Report;
    }
    if lower.contains("presentation") || lower.ends_with(".pptx") || lower.ends_with(".ppt") {
        return AttachmentCategory::Presentation;
    }
    if lower.ends_with(".xlsx") || lower.ends_with(".xls") || lower.ends_with(".csv") {
        return AttachmentCategory::Spreadsheet;
    }
    if is_source_code_filename(&lower) {
        return AttachmentCategory::SourceCode;
    }
    if is_archive_filename(&lower) {
        return AttachmentCategory::Archive;
    }
    if content_type.starts_with("image/") {
        if lower.contains("screenshot") || lower.contains("screen") {
            return AttachmentCategory::Screenshot;
        }
        return AttachmentCategory::Image;
    }
    if content_type == "application/pdf" {
        return AttachmentCategory::Report;
    }

    AttachmentCategory::Unknown
}

fn is_source_code_filename(filename: &str) -> bool {
    [".rs", ".py", ".js", ".ts", ".go", ".java", ".c", ".cpp"]
        .iter()
        .any(|extension| filename.ends_with(extension))
}

fn is_archive_filename(filename: &str) -> bool {
    [".zip", ".rar", ".7z", ".tar.gz", ".tar"]
        .iter()
        .any(|extension| filename.ends_with(extension))
}
