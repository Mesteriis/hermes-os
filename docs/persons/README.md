# Hermes Hub — Persons Module

Relationship Intelligence System. Persons — это не адресная книга, а долговременная память о людях и отношениях.

## Ключевые возможности

- **Multi-Channel Identity** — один человек, много каналов (email, Telegram, GitHub, LinkedIn...)
- **Identity Resolution** — склейка дублей, merge/split с confidence-скорингом
- **Person Memory** — факты, карточки памяти, предпочтения с source/confidence
- **Relationship Timeline** — история отношений, события, milestones
- **Communication DNA** — стиль общения, язык, тон, verbosity, паттерны ответов
- **Enrichment Engine** — обогащение из GitHub, LinkedIn, публичного веба
- **Expertise & Skills** — поиск людей по навыкам и доменам
- **Trust & Reliability** — обещания, риски, trust score
- **Relationship Health** — мониторинг активности, watchlist, health status
- **Person Investigator** — AI-ассистент: досье, подготовка к встречам
- **Analytics & Export** — relationship score, heatmap, интеллектуальный score, Markdown/JSON export

## Состояние

| Метрика | Значение |
|---|---|
| Модулей | 13 |
| Таблиц | 14 |
| API endpoint'ов | 35 |
| Миграций | 4 |
| ADR | 5 |
| Строк кода | ~4000 |
| Реализовано разделов спеки | 63 из 83 |
| Не в scope persons | 8 |
| Deferred | 12 |

## Навигация

- [API Reference](api.md) — все 35 endpoint
- [Статус реализации](status.md) — детальный статус по разделам спеки
- [Модель данных](data-model.md) — 14 таблиц со схемой
- [Архитектура](architecture.md) — модули и потоки данных
