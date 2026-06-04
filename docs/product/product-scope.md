# Product Scope

## In Scope

### Communications

- Email
- Telegram
- WhatsApp
- SMS как опциональный future channel
- единая коммуникационная лента
- threading, participants, attachments, delivery metadata
- spam, marketing и relevance classification

### Contacts

- люди и организации
- каналы связи
- история коммуникаций
- отношения между контактами
- связь с документами, проектами, задачами и событиями
- identity resolution и merge workflow

### Knowledge Graph

- Person
- Company
- Project
- Document
- Message
- Event
- Task
- Meeting
- Location
- Organization
- отношения как полноценные объекты с provenance

### Tasks

- извлечение задач из коммуникаций
- дедлайны, напоминания, исполнители и статусы
- связь задач с источниками и проектами
- audit trail изменения статуса

### Documents

- PDF
- Office
- images
- Markdown
- OCR
- summary
- entity extraction
- linking
- versioning

### Search

- full text search
- semantic search
- graph-aware memory queries
- source-backed answers

### AI Agents

- HESTIA - главный координатор
- HERMES - коммуникации
- MNEMOSYNE - память
- ATHENA - аналитика
- HEPHAESTUS - разработка и tool automation

## Out of Scope For Initial Implementation, But Architecturally Supported

- multi-user SaaS
- enterprise CRM workflows
- public API marketplace
- global cloud sync as required dependency
- end-to-end encrypted multi-device sync
- autonomous external actions without explicit permission policy

Эти пункты не должны диктовать первую реализацию, но архитектура не должна закрывать дорогу к ним.

## Non-Goals

- заменить Gmail, Telegram или WhatsApp как network provider
- обучать персональную LLM на приватных данных
- хранить только embeddings без исходников
- скрывать автоматические решения без provenance
- создавать одну большую универсальную таблицу активности

## Capability Map

| Capability | Core entities | Primary output |
| --- | --- | --- |
| Communication ingestion | Message, Event, Contact, Attachment | normalized events and messages |
| Contact memory | Person, Company, Relationship | contact timeline and graph |
| Document understanding | Document, Version, EntityMention | indexed and linked documents |
| Task extraction | Task, Event, Person, Project | actionable task graph |
| Search and recall | Message, Document, Event, Entity | ranked source-backed results |
| AI orchestration | Agent, Tool, MemoryQuery | explainable AI workflows |
| Project memory | Project, Task, Document, Event | project timeline and decisions |
