# Email Channel — Architectural Blockers

Явно задокументированные блокеры с причинами и планом решения.
API: `GET /api/v1/communications/blockers`

These blockers apply to the current email-channel implementation. Cross-channel
Communications, Obligations, Decisions and Polygraph work is tracked in
`../refactoring/implementation-alignment-plan.md`.

## 1. §8 — Безопасность вложений (sandbox, антивирус)

**Причина**: Требует внешних инструментов — ClamAV, контейнеризированная песочница, OLE-парсер макросов. Это инфраструктурная задача, а не кодовая.

**План**: Интегрировать ClamAV как sidecar-контейнер в `docker-compose.yml`, добавить `attachment_scanner` с реальной имплементацией, заменить `not_scanned` на `clean/suspicious/malicious`.

## 2. §12 — Криптографическая верификация подписей

**Причина**: Требует OpenSSL, GPG, КриптоПро SDK. Это внешние нативные библиотеки (C/C++), не Rust-крейты. Нужна отдельная интеграционная работа с FFI или вызовом CLI.

**План**: Создать `email_crypto` модуль с привязкой к OpenSSL/GPG. Сертификаты из macOS Keychain читать через Security framework. ГОСТ-подписи — через КриптоПро CLI или отдельный микросервис.

## 3. §16-17 — Outbox tracking и Follow-up engine

**Причина**: Требует DSN (Delivery Status Notification) / MDN (Message Disposition Notification) парсинга из входящих уведомлений о доставке, а также SMTP-колбеков/webhook'ов от провайдера. Это асинхронный event-driven flow.

**План**: Реализовать DSN/MDN парсер (RFC 3464/3798), добавить фоновый воркер для отслеживания статусов отправленных писем по Message-ID, создать таблицу `email_outbox_tracking`.

## 4. §28-29 — Интеграции и массовые действия

**Причина**: Каждая интеграция (Jira, YouTrack, Google Calendar, Apple Notes, Obsidian) — отдельный коннектор со своим API и аутентификацией. Массовые действия требуют batch API и очередей задач.

**План**: Реализовать как plugin-коннекторы по образцу существующих Telegram/WhatsApp модулей. Массовые действия — через фоновые задачи projection runner.

## 5. §8.2 — Безопасная распаковка архивов

**Причина**: Требует потоковой распаковки с защитой от zip bomb, path traversal, вложенных архивов. Нужна интеграция с zip/rar/7z крейтами и настройка лимитов размера, глубины, количества файлов.

**План**: Создать `email_archive_extractor` модуль с лимитами: max 100MB архив, max 1GB распакованных, max глубина 3, max 1000 файлов. Использовать крейты `zip` + `sevenz-rust`.

## 6. §9.3 — OCR (распознавание текста)

**Причина**: Требует Tesseract OCR или облачного OCR-сервиса. Это тяжёлая зависимость (50+ MB trained data для каждого языка).

**План**: Опциональная фича под feature-флагом `ocr`. Добавить `tesseract-rs` крейт. Без флага — только извлечение текста из PDF/DOCX через существующие парсеры.

## Не-блокеры (не входят в scope email-модуля)

Следующие разделы спецификации не являются частью email-модуля и реализуются отдельно:

- **Exchange/Fastmail/Proton/Maildir адаптеры** (§3) — отдельные provider adapter'ы
- **Rich-редактор шаблонов в UI** (§31) — задача фронтенда, API готово
- **Импорт EML/MBOX через UI** (§30) — задача фронтенда, бекенд готов
- **Undo-send** (§4.2) — зависит от §16 (outbox tracking)
