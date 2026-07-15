### Summary / Резюме

Добавить в русскую wiki страницу `operations/configuration.md` документацию по конфигурационному файлу `.cargo/config.toml` — описать его назначение и все четыре псевдонима (`hermes-nextest`, `hermes-nextest-ci`, `hermes-nextest-integration`, `hermes-llvm-cov`), упрощающих запуск тестов и анализ покрытия в проекте `hermes-hub`.

### Proposed pages / Предлагаемые страницы

**`operations/configuration.md`**

```markdown
---
tags: [configuration, cargo, hermes]
---

# Конфигурация Cargo

Файл `.cargo/config.toml` определяет псевдонимы (aliases) для `cargo`, которые стандартизируют запуск тестов через `cargo-nextest` и генерацию LLVM-отчётов о покрытии. Все псевдонимы используют манифест `backend/Cargo.toml`, чтобы явно указывать рабочую область бэкенда.

## Псевдонимы

### `hermes-nextest`

Запускает все тесты проекта с помощью `cargo-nextest`.

```sh
cargo hermes-nextest
```

Раскрывается в команду:

```
nextest run --manifest-path backend/Cargo.toml
```

### `hermes-nextest-ci`

Запускает тесты в профиле `ci`, оптимизированном для конвейеров непрерывной интеграции (меньше лишнего вывода, повторяемые результаты).

```sh
cargo hermes-nextest-ci
```

Раскрывается в команду:

```
nextest run --manifest-path backend/Cargo.toml --profile ci
```

### `hermes-nextest-integration`

Запускает только интеграционные тесты (помеченные `#[cfg(test)]` в интеграционных файлах) с профилем `integration`.

```sh
cargo hermes-nextest-integration
```

Раскрывается в команду:

```
nextest run --manifest-path backend/Cargo.toml --profile integration --tests
```

Флаг `--tests` ограничивает запуск только тестовыми целями, исключая бенчмарки и примеры.

### `hermes-llvm-cov`

Генерирует отчёт о покрытии кода с помощью `cargo-llvm-cov` (инструмент на базе LLVM source-based code coverage).

```sh
cargo hermes-llvm-cov
```

Раскрывается в команду:

```
llvm-cov --manifest-path backend/Cargo.toml
```

## Исходный файл

`.cargo/config.toml` (328 байт)
```

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `.cargo/config.toml` | Определение четырёх cargo-алиасов: `hermes-nextest`, `hermes-nextest-ci`, `hermes-nextest-integration`, `hermes-llvm-cov`; их точные раскрытия; путь к манифесту `backend/Cargo.toml`; используемые профили (`ci`, `integration`) и флаг `--tests`. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста (единственный файл `.cargo/config.toml`) расхождения кода, документации или ADR не видны.
