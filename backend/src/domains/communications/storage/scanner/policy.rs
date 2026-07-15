use super::*;

pub(super) fn normalized_extension(filename: Option<&str>) -> Option<String> {
    let filename = filename?.trim();
    let basename = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .trim();
    let (_, extension) = basename.rsplit_once('.')?;
    let extension = extension.trim().to_ascii_lowercase();
    (!extension.is_empty()).then_some(extension)
}

pub(super) fn normalized_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase()
}

pub(super) fn has_executable_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(b"MZ") || bytes.starts_with(b"\x7fELF")
}

pub(super) fn has_legacy_ole_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(LEGACY_OLE_MAGIC)
}

pub(super) fn is_uninspected_archive(
    extension: Option<&str>,
    content_type: &str,
    bytes: &[u8],
) -> bool {
    matches!(extension, Some("rar" | "7z"))
        || matches!(
            content_type,
            "application/vnd.rar" | "application/x-rar-compressed" | "application/x-7z-compressed"
        )
        || bytes.starts_with(RAR4_MAGIC)
        || bytes.starts_with(RAR5_MAGIC)
        || bytes.starts_with(SEVEN_Z_MAGIC)
}

pub(super) fn is_ooxml_document(extension: Option<&str>, content_type: &str) -> bool {
    matches!(
        extension,
        Some("docx" | "xlsx" | "pptx" | "docm" | "xlsm" | "pptm")
    ) || matches!(
        content_type,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            | "application/vnd.ms-word.document.macroenabled.12"
            | "application/vnd.ms-excel.sheet.macroenabled.12"
            | "application/vnd.ms-powerpoint.presentation.macroenabled.12"
    )
}

pub(super) enum OoxmlInspection {
    NoMacroPayload,
    MacroPayload,
    Unreadable,
    TooLarge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LegacyOleInspection {
    MacroMarkers,
    NoMacroMarkers,
    Unreadable,
    TooLarge,
}

pub(super) fn inspect_ooxml_container(bytes: &[u8]) -> OoxmlInspection {
    if bytes.len() > MAX_OOXML_INSPECTION_BYTES {
        return OoxmlInspection::TooLarge;
    }

    let mut archive = match ZipArchive::new(Cursor::new(bytes)) {
        Ok(archive) => archive,
        Err(_) => return OoxmlInspection::Unreadable,
    };
    if archive.len() > MAX_OOXML_INSPECTION_ENTRIES {
        return OoxmlInspection::TooLarge;
    }

    for index in 0..archive.len() {
        let file = match archive.by_index(index) {
            Ok(file) => file,
            Err(_) => return OoxmlInspection::Unreadable,
        };
        let name = file.name().to_ascii_lowercase();
        if name.ends_with("vbaproject.bin") || name.ends_with("vbadata.xml") {
            return OoxmlInspection::MacroPayload;
        }
    }
    OoxmlInspection::NoMacroPayload
}

/// Inspects only CFB allocation metadata and directory entry names. It never follows document
/// content streams, expands embedded objects, or writes a file to disk.
pub(super) fn inspect_legacy_ole_container(bytes: &[u8]) -> LegacyOleInspection {
    if bytes.len() > MAX_OOXML_INSPECTION_BYTES {
        return LegacyOleInspection::TooLarge;
    }
    let Some(header) = bytes.get(..512) else {
        return LegacyOleInspection::Unreadable;
    };
    if !header.starts_with(LEGACY_OLE_MAGIC)
        || read_u16(header, 28) != Some(0xFFFE)
        || read_u16(header, 32) != Some(6)
    {
        return LegacyOleInspection::Unreadable;
    }
    let Some(sector_shift) = read_u16(header, 30) else {
        return LegacyOleInspection::Unreadable;
    };
    if !matches!(sector_shift, 9 | 12) {
        return LegacyOleInspection::Unreadable;
    }
    let sector_size = 1_usize << sector_shift;
    let Some(payload_len) = bytes.len().checked_sub(512) else {
        return LegacyOleInspection::Unreadable;
    };
    if payload_len == 0 || payload_len % sector_size != 0 {
        return LegacyOleInspection::Unreadable;
    }
    let sector_count = payload_len / sector_size;
    let Some(fat_sector_count) = read_u32(header, 44).map(|value| value as usize) else {
        return LegacyOleInspection::Unreadable;
    };
    if fat_sector_count == 0 || fat_sector_count > 109 {
        // The header can reference more FAT sectors through DIFAT chains. Refuse that complex
        // case rather than attempting unbounded traversal in the synchronous safety scanner.
        return LegacyOleInspection::Unreadable;
    }
    let Some(first_directory_sector) = read_u32(header, 48) else {
        return LegacyOleInspection::Unreadable;
    };
    if !is_legacy_ole_sector_id(first_directory_sector, sector_count) {
        return LegacyOleInspection::Unreadable;
    }

    let mut fat = Vec::with_capacity(fat_sector_count * (sector_size / 4));
    for index in 0..fat_sector_count {
        let Some(fat_sector_id) = read_u32(header, 76 + index * 4) else {
            return LegacyOleInspection::Unreadable;
        };
        if !is_legacy_ole_sector_id(fat_sector_id, sector_count) {
            return LegacyOleInspection::Unreadable;
        }
        let Some(sector) = legacy_ole_sector(bytes, sector_size, fat_sector_id) else {
            return LegacyOleInspection::Unreadable;
        };
        for entry in sector.chunks_exact(4) {
            let Some(value) = read_u32(entry, 0) else {
                return LegacyOleInspection::Unreadable;
            };
            fat.push(value);
        }
    }

    let mut sector_id = first_directory_sector;
    let mut visited = HashSet::new();
    for _ in 0..MAX_LEGACY_OLE_DIRECTORY_SECTORS {
        if !visited.insert(sector_id) {
            return LegacyOleInspection::Unreadable;
        }
        let Some(sector) = legacy_ole_sector(bytes, sector_size, sector_id) else {
            return LegacyOleInspection::Unreadable;
        };
        for entry in sector.chunks_exact(128) {
            match legacy_ole_directory_name(entry) {
                Ok(Some(name)) if is_legacy_ole_macro_marker(&name) => {
                    return LegacyOleInspection::MacroMarkers;
                }
                Ok(_) => {}
                Err(()) => return LegacyOleInspection::Unreadable,
            }
        }
        let Some(next_sector) = fat.get(sector_id as usize).copied() else {
            return LegacyOleInspection::Unreadable;
        };
        if next_sector == LEGACY_OLE_END_OF_CHAIN {
            return LegacyOleInspection::NoMacroMarkers;
        }
        if !is_legacy_ole_sector_id(next_sector, sector_count) {
            return LegacyOleInspection::Unreadable;
        }
        sector_id = next_sector;
    }
    LegacyOleInspection::Unreadable
}

pub(super) fn legacy_ole_sector(bytes: &[u8], sector_size: usize, sector_id: u32) -> Option<&[u8]> {
    let offset = 512_usize.checked_add((sector_id as usize).checked_mul(sector_size)?)?;
    bytes.get(offset..offset.checked_add(sector_size)?)
}

pub(super) fn is_legacy_ole_sector_id(sector_id: u32, sector_count: usize) -> bool {
    sector_id != LEGACY_OLE_END_OF_CHAIN
        && sector_id != LEGACY_OLE_FREE_SECTOR
        && sector_id != LEGACY_OLE_FAT_SECTOR
        && (sector_id as usize) < sector_count
}

pub(super) fn legacy_ole_directory_name(entry: &[u8]) -> Result<Option<String>, ()> {
    if entry.len() != 128 || entry.get(66).copied() == Some(0) {
        return Ok(None);
    }
    let name_len = read_u16(entry, 64).ok_or(())? as usize;
    if !(2..=64).contains(&name_len) || !name_len.is_multiple_of(2) {
        return Err(());
    }
    let name_bytes = entry.get(..name_len - 2).ok_or(())?;
    let mut code_units = Vec::with_capacity(name_bytes.len() / 2);
    for pair in name_bytes.chunks_exact(2) {
        code_units.push(u16::from_le_bytes([pair[0], pair[1]]));
    }
    String::from_utf16(&code_units).map(Some).map_err(|_| ())
}

pub(super) fn is_legacy_ole_macro_marker(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "vba" | "_vba_project_cur" | "vba_project" | "macros"
    )
}

pub(super) fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let bytes = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub(super) fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let bytes = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

pub(super) fn is_active_content_extension(extension: &str) -> bool {
    matches!(
        extension,
        "app"
            | "bat"
            | "cmd"
            | "com"
            | "dll"
            | "dmg"
            | "exe"
            | "hta"
            | "jar"
            | "jse"
            | "js"
            | "msi"
            | "ps1"
            | "scr"
            | "vbe"
            | "vbs"
            | "wsf"
    )
}

pub(super) fn is_macro_document_extension(extension: &str) -> bool {
    matches!(
        extension,
        "docm" | "dotm" | "xlsm" | "xltm" | "pptm" | "potm"
    )
}

pub(super) fn is_mime_extension_mismatch(content_type: &str, extension: &str) -> bool {
    let expected = expected_extensions_for_content_type(content_type);
    !expected.is_empty() && !expected.contains(&extension)
}

pub(super) fn expected_extensions_for_content_type(content_type: &str) -> &'static [&'static str] {
    match content_type {
        "application/pdf" => &["pdf"],
        "application/zip" => &["zip"],
        "application/vnd.rar" | "application/x-rar-compressed" => &["rar"],
        "application/x-7z-compressed" => &["7z"],
        "image/jpeg" => &["jpg", "jpeg"],
        "image/png" => &["png"],
        "text/csv" => &["csv"],
        "text/plain" => &["txt", "text", "log", "csv"],
        _ => &[],
    }
}
