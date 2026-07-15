---
chunk_id: 019-config-backend
batch_id: batch-20260628T214902
group: backend
role: config
source_status: pending
source_count: 3
generated_by: code-wiki-ru
---

# 019-config-backend — backend/config

- Target index: [[operations/configuration]]
- Batch: `batch-20260628T214902`
- Source files: `3`

## Резюме

Документация раздела `operations/configuration.md` русской Obsidian‑wiki отсутствовала.
На основе встроенных файлов `Cargo.toml` (фичи и опциональные зависимости) и `system_sources.toml` (перечень источников с их свойствами) создана страница «Конфигурация бэкенда».
Она описывает сборочные возможности (features) и схему системных источников, пригодную для ручной настройки и понимания текущей конфигурации.

## Предложенные страницы

**`operations/configuration.md`**

```markdown
# Конфигурация бэкенда

## Сборочные возможности (features)

Файл `backend/Cargo.toml` определяет следующие feature‑флаги:

| Feature                           | Состав / поведение (из контекста)                                                                 |
|-----------------------------------|---------------------------------------------------------------------------------------------------|
| `default`                         | Пустой набор. Дополнительные зависимости не включаются.                                           |
| `test-support`                    | Предназначен для тестовой поддержки. Детали реализации в данном контексте не раскрыты.            |
| `whatsapp-native-md-runtime`      | Включает опциональные зависимости `wa-rs` и `wa-rs‑core` (версии 0.2.0).                          |
| `whatsapp-business-cloud-runtime` | Собственных зависимостей не добавляет; поведение, вероятно, реализовано через условную компиляцию. |

Опциональные (optional) зависимости, управляемые feature‑флагами:
- `wa-rs` (опционально, активируется `whatsapp-native-md-runtime`)
- `wa-rs‑core` (опционально, активируется `whatsapp-native-md-runtime`)

## Источники данных (system_sources)

Конфигурация источников хранится в файле `backend/fixtures/signal_hub/system_sources.toml`.
Массив ```sources``` содержит записи со следующими полями:

| Поле                  | Тип    | Описание                                                                 |
|-----------------------|--------|--------------------------------------------------------------------------|
| `code`                | строка | Уникальный код источника                                                 |
| `display_name`        | строка | Отображаемое имя                                                         |
| `category`            | строка | Группа: `system`, `intelligence`, `communications`, `code`, и т.д.       |
| `source_kind`         | строка | Тип: `system`, `provider`, `local`, `fixture`                            |
| `default_enabled`     | bool   | Источник включён по умолчанию                                            |
| `supports_connections`| bool   | Поддерживает настройку внешних соединений                                |
| `supports_runtime`    | bool   | Может работать в реальном времени                                        |
| `supports_replay`     | bool   | Поддерживает воспроизведение (replay)                                    |
| `supports_pause`      | bool   | Можно приостановить                                                      |
| `supports_mute`       | bool   | Можно заглушить                                                          |

### Определённые источники

| code              | display_name       | category        | source_kind | connections | runtime |
|-------------------|--------------------|-----------------|-------------|-------------|---------|
| `system`          | System             | system          | system      | нет         | да      |
| `ai`              | AI                 | intelligence    | system      | нет         | да      |
| `mail`            | Mail               | communications  | provider    | да          | да      |
| `telegram`        | Telegram           | communications  | provider    | да          | да      |
| `whatsapp`        | WhatsApp           | communications  | provider    | да          | да      |
| `zoom`            | Zoom               | communications  | provider    | да          | да      |
| `github`          | GitHub             | code            | provider    | да          | да      |
| `browser`         | Browser            | capture         | local       | нет         | да      |
| `rss`             | RSS                | knowledge       | provider    | да          | да      |
| `calendar`        | Calendar           | time            | provider    | да          | да      |
| `filesystem`      | Filesystem         | documents       | local       | нет         | да      |
| `home_assistant`  | Home Assistant     | environment     | provider    | да          | да      |
| `voice`           | Voice              | capture         | local       | нет         | да      |
| `fixture`         | Fixture            | test            | fixture     | нет         | нет     |

Все источники, кроме `fixture`, имеют `supports_runtime = true`.
У каждого источника `supports_replay`, `supports_pause` и `supports_mute` равны `true`.

## См. также

- `Cargo.toml` — полный список зависимостей и бинарных целей
- `system_sources.toml` — эталонный перечень источников
```

## Покрытие источников

1. **`backend/Cargo.toml`**
   - Перечень сборочных фич (`[features]`): `default`, `test-support`, `whatsapp-native-md-runtime`, `whatsapp-business-cloud-runtime`.
   - Условные опциональные зависимости `wa-rs` и `wa-rs-core`, управляемые фичами.
   - Структура документа `[features]` (включена на уровне описания, а не дословно).

2. **`backend/fixtures/signal_hub/system_sources.toml`**
   - Все 14 записей массива ```sources``` с полными значениями полей: `code`, `display_name`, `category`, `source_kind`, `default_enabled`, `supports_connections`, `supports_runtime`, `supports_replay`, `supports_pause`, `supports_mute`.
   - Общие свойства: единообразие флагов `supports_replay/pause/mute` и исключение `fixture` из `supports_runtime`.

3. **`backend/Cargo.lock`** (файл блокировки зависимостей)
   - Непосредственно на странице конфигурации не покрыт — его содержимое не влияет на документируемую схему.

## Исходные файлы

- [`backend/Cargo.lock`](../../../../backend/Cargo.lock)
- [`backend/Cargo.toml`](../../../../backend/Cargo.toml)
- [`backend/fixtures/signal_hub/system_sources.toml`](../../../../backend/fixtures/signal_hub/system_sources.toml)

## Кандидаты на drift

- **Несоответствие WhatsApp runtime ↔ source**
  `Cargo.toml` различает два runtime для WhatsApp (`whatsapp-native-md-runtime` и `whatsapp-business-cloud-runtime`), тогда как `system_sources.toml` определяет единый источник `whatsapp` без поля выбора runtime.
  Возможен drift: либо конфигурация sources должна отражать конкретный runtime, либо выбор runtime осуществляется иным способом, не отражённым в предоставленных файлах.
  Без дополнительного контекста утверждать окончательно нельзя.

- **Отсутствующая документация фичи `test-support`**
  В предоставленном окружении `Cargo.toml` объявляет флаг `test-support`, но его использование (в том числе в тестах или условной компиляции) не подтверждено ни одним из встроенных файлов. Возможен неполный охват документации.
