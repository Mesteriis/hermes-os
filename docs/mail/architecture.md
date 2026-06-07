# Hermes Mail — Архитектура

## Слои

```
┌─────────────────────────────────────────────┐
│                  UI (SvelteKit)              │
├─────────────────────────────────────────────┤
│              HTTP API (lib.rs)               │
│  50+ endpoint'ов через axum::Router          │
├─────────────────────────────────────────────┤
│         Domain Services (36 модулей)         │
│  intelligence │ threads │ rules │ templates  │
│  drafts │ finance │ legal │ signatures      │
│  analytics │ search │ extract │ personas    │
├─────────────────────────────────────────────┤
│        Storage Layer                         │
│  PostgreSQL (metadata) + Tantivy (search)    │
│  + LocalFS blob storage (docker/data/mail/) │
├─────────────────────────────────────────────┤
│        Provider Adapters                     │
│  Gmail API │ IMAP │ SMTP │ Ollama (LLM)     │
└─────────────────────────────────────────────┘
```

## Поток данных

### Входящие (ingestion)
```
Provider → Raw Records → Message Projection → Auto-Analysis → Graph → Search Index
```

### Исходящие (sending)
```
Draft → SMTP Client → Provider
```

### Анализ
```
Message → Heuristic Engine → AI Category + Score → Workflow State
         → LLM (Ollama) → Summary + Classification
         → SPF/DKIM/DMARC Parser → Auth Risk
         → Signature Detector → S/MIME + PGP status
```

## Ключевые ADR

| ADR | Тема |
|---|---|
| ADR-0001 | Event sourcing — spine системы |
| ADR-0005 | PostgreSQL — primary store |
| ADR-0006 | Tantivy — full-text search |
| ADR-0009 | Ollama — локальный AI |
| ADR-0041 | Email provider ingestion foundation |
| ADR-0042 | Secret references для credentials |
| ADR-0044 | Account setup + encrypted vault |
| ADR-0046 | Blob storage для вложений |
| ADR-0053 | Database encrypted vault |
| ADR-0055 | Full read-write email networking |

## База данных

12 таблиц для почтового модуля:
- `communication_messages` — основная таблица писем (workflow_state, ai_category, importance_score, ...)
- `communication_attachments` — вложения
- `communication_mail_blobs` — blob-хранилище
- `email_rules` — правила автоматизации
- `email_templates` — шаблоны писем
- `email_personas` — личности отправки
- `email_drafts` — черновики
- `email_invoices` — счета
- `email_legal_documents` — юрдокументы
- `email_certificates` — сертификаты
- `communication_provider_accounts` — аккаунты провайдеров
- `communication_raw_records` — сырые записи (append-only)
