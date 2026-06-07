# Hermes Hub — Organizations Module

Organizations — это не поле `company` у контакта, а самостоятельные сущности памяти. Модуль хранит организации, их идентичности, контакты, документы, контракты, порталы, процедуры и историю взаимодействий.

## Ключевые возможности

- **Organization as first-class entity** — организация не поле контакта, а отдельная сущность
- **Multi-identity model** — домены, VAT/CIF/NIF, GitHub org, LinkedIn, порталы
- **Identity resolution** — склейка дублей по доменам, VAT, названиям
- **Contact links** — many-to-many persons↔organizations с ролью и отделом
- **Organization memory** — факты, карточки памяти, предпочтения с source/confidence
- **Timeline** — история взаимодействий с организацией
- **Portals, procedures, playbooks** — порталы, типовые процедуры, автоматизируемые сценарии
- **Finance hub** — контракты, compliance, сервисы, продукты, банковские реквизиты
- **Passive OSINT enrichment** — VIES, GitHub, LinkedIn, публичные реестры
- **Health & watchtower** — мониторинг состояния, риски, алерты
- **Investigator** — досье, brief, context pack

## Состояние

| Метрика | Значение |
|---|---|
| Модулей | 8 |
| Таблиц | 27 |
| API endpoint'ов | 28 |
| Миграций | 6 |
| ADR | 6 |

## Навигация

- [API Reference](api.md)
- [Модель данных](data-model.md)
- [Архитектура](architecture.md)
