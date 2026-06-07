# Hermes Mail — Описание модулей

## Core Pipeline

| Модуль | Файл | Назначение |
|---|---|---|
| Email Ingestion | `email_ingestion.rs` | Точка входа авто-анализа. Каждое входящее письмо проходит через Hermes. |
| Email Sync | `email_sync.rs` | Планирование синхронизации (Gmail/IMAP/iCloud) |
| Email Sync Pipeline | `email_sync_pipeline.rs` | Полный пайплайн: парсинг → проекция → анализ → вложения |
| Email RFC822 | `email_rfc822.rs` | Парсинг RFC 2822 / MIME |
| Email Import | `email_import.rs` | Импорт фикстур |
| Email Fixture Pipeline | `email_fixture_pipeline.rs` | Пайплайн для тестовых данных |
| Email Fixture Export | `email_fixture_export.rs` | Экспорт фикстур |
| Messages | `messages.rs` | Проекция сообщений. 8 workflow-состояний, AI-поля, фильтры |

## Intelligence & Analysis

| Модуль | Назначение |
|---|---|
| `email_intelligence.rs` | AI-анализ: скоринг (0-100), 13 категорий, LLM-классификация |
| `email_spf_dkim.rs` | SPF/DKIM/DMARC парсинг заголовков, оценка риска спуфинга |
| `email_explain.rs` | "Почему это письмо важно?" + умные подсказки CC |
| `email_multilingual.rs` | Определение языка (ru/en/es/de/uk/zh), перевод через LLM |
| `email_extract.rs` | Извлечение задач и заметок из письма (LLM + эвристики) |

## Organization & Workflow

| Модуль | Назначение |
|---|---|
| `email_threads.rs` | Группировка писем в треды, метрики здоровья |
| `email_flags.rs` | Pin / Snooze / Label / Mute — флаги в JSONB метаданных |
| `email_subscriptions.rs` | Детектор рассылок, поиск unsubscribe |
| `email_analytics.rs` | Дашборд здоровья ящика, топ отправителей |

## Outgoing

| Модуль | Назначение |
|---|---|
| `email_send.rs` | SMTP клиент (EHLO, AUTH LOGIN, MAIL FROM, RCPT TO, DATA) |
| `email_drafts.rs` | Черновики: 5 статусов, авто-устаревание |
| `email_actions.rs` | Reply/Forward/reply-all с цитированием, EML-forward |
| `email_ai_reply.rs` | AI генератор ответов: выбор тона и языка, варианты |
| `email_templates.rs` | Шаблоны с подстановкой переменных |
| `email_rich_template.rs` | Rich-шаблоны: conditional, table, button, divider |
| `email_personas.rs` | Несколько личностей отправки на аккаунт |

## Documents & Finance

| Модуль | Назначение |
|---|---|
| `email_finance.rs` | Счета: 7 статусов, суммы, валюты, контрагенты |
| `email_legal.rs` | Юрдокументы: 11 типов (contract/NDA/MSA/DPA/...), 6 статусов |

## Security & Trust

| Модуль | Назначение |
|---|---|
| `email_signatures.rs` | Сертификаты: 7 типов, 9 провайдеров, детекция S/MIME+PGP |
| `attachment_intelligence.rs` | Классификация вложений: 15 категорий, уровни риска |
| `email_attachment_dedup.rs` | Поиск дубликатов по SHA-256 + похожие имена |

## Automation

| Модуль | Назначение |
|---|---|
| `email_rules.rs` | Движок правил: field/operator/value, 4 режима выполнения |

## Search & Export

| Модуль | Назначение |
|---|---|
| `email_search.rs` | Мост к Tantivy: индексация и поиск писем |
| `email_export.rs` | Экспорт письма в EML / Markdown / JSON |

## Provider Networking

| Модуль | Назначение |
|---|---|
| `email_provider_network.rs` | Gmail API + IMAP клиенты (read path) |
| `email_imap_write.rs` | IMAP write операции (STORE, EXPUNGE) |
| `email_account_setup.rs` | Настройка аккаунтов: Gmail OAuth, IMAP, шифрование |

## Support

| Модуль | Назначение |
|---|---|
| `email_sources.rs` | Типы источников писем |
| `mail_storage.rs` | Blob-хранилище вложений (LocalFS) |
| `email_blockers.rs` | Документирование архитектурных блокеров |
