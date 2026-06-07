//! # Hermes Mail — Explicit Architecture Blockers
//!
//! Sections that are NOT implemented and WHY.
//! This file serves as authoritative documentation of known gaps.

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ArchitectureBlocker {
    pub section: String,
    pub feature: String,
    pub reason: String,
    pub resolution: String,
}

/// Returns all known blockers with explanations.
pub fn list_blockers() -> Vec<ArchitectureBlocker> {
    vec![
        ArchitectureBlocker {
            section: "§8".into(),
            feature: "Безопасность вложений (sandbox, антивирус)".into(),
            reason: "Требует внешних инструментов: ClamAV, контейнеризированная песочница, OLE-парсер макросов. Это инфраструктурная задача, не кодовая.".into(),
            resolution: "Интегрировать ClamAV как sidecar-контейнер в docker-compose, добавить attachment_scanner с реальной имплементацией.".into(),
        },
        ArchitectureBlocker {
            section: "§12 (крипто-проверка)".into(),
            feature: "Реальная криптографическая верификация подписей (S/MIME, PGP, CAdES, XAdES, ГОСТ)".into(),
            reason: "Требует OpenSSL, GPG, КриптоПро SDK. Это внешние нативные библиотеки, не Rust-крейты. Нужна отдельная интеграционная работа.".into(),
            resolution: "Добавить email_crypto модуль с привязкой к OpenSSL/GPG через FFI или CLI. Сертификаты из Keychain читать через macOS Security framework.".into(),
        },
        ArchitectureBlocker {
            section: "§16-17".into(),
            feature: "Outbox tracking (delivery status, read receipts, bounce detection) и Follow-up engine".into(),
            reason: "Требует DSN/MDN парсинга из входящих уведомлений о доставке, а также SMTP-колбеков/webhook'ов от провайдера. Это асинхронный event-driven flow, требующий постоянного мониторинга входящих.".into(),
            resolution: "Реализовать DSN/MDN парсер, добавить фоновый воркер для отслеживания статусов отправленных писем по Message-ID.".into(),
        },
        ArchitectureBlocker {
            section: "§28-29".into(),
            feature: "Интеграции (Jira, YouTrack, Google Calendar, Apple Notes, Obsidian) и массовые действия".into(),
            reason: "Каждая интеграция — отдельный коннектор со своим API и аутентификацией. Это отдельные модули, не часть email-подсистемы. Массовые действия требуют batch API и очередей.".into(),
            resolution: "Реализовать как plugin-коннекторы по образцу Telegram/WhatsApp модулей. Массовые действия — через фоновые задачи projection runner.".into(),
        },
        ArchitectureBlocker {
            section: "§8.2".into(),
            feature: "Безопасная распаковка архивов (zip slip protection, вложенные архивы, password detection)".into(),
            reason: "Требует потоковой распаковки с защитой от zip bomb/path traversal. Нужна интеграция с zip/rar/7z крейтами и настройка лимитов.".into(),
            resolution: "Добавить email_archive_extractor с лимитами размера/глубины/количества файлов, использовать крейт zip + rar + sevenz-rs.".into(),
        },
        ArchitectureBlocker {
            section: "§9.3".into(),
            feature: "OCR (распознавание текста из PDF-сканов и изображений)".into(),
            reason: "Требует Tesseract OCR или облачного OCR-сервиса. Это тяжёлая зависимость (50+ MB trained data).".into(),
            resolution: "Опциональная фича: добавить tesseract-rs крейт под feature-флагом. Без неё — только текст из PDF/DOCX.".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_blockers_have_reason_and_resolution() {
        for b in list_blockers() {
            assert!(!b.reason.is_empty(), "{} has no reason", b.section);
            assert!(!b.resolution.is_empty(), "{} has no resolution", b.section);
        }
    }
    #[test]
    fn blocker_count_is_stable() {
        assert_eq!(
            list_blockers().len(),
            6,
            "Expected exactly 6 architectural blockers"
        );
    }
}
