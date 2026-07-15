### Summary / Резюме

Добавить на страницу `operations/configuration.md` описание файла `.cargo/config.toml` и определённых в нём псевдонимов Cargo. Страница ранее отсутствовала, поэтому создаётся с нуля на основе предоставленного контекста.

### Proposed pages / Предлагаемые страницы

#### operations/configuration.md

```markdown
# Конфигурация

## `.cargo/config.toml`

Файл `.cargo/config.toml` задаёт псевдонимы (aliases) для запуска инструментов сборки и тестирования.

### Псевдонимы

| Псевдоним | Выполняемая команда |
|---|---|
| `hermes-nextest` | `nextest run --manifest-path backend/Cargo.toml` |
| `hermes-nextest-ci` | `nextest run --manifest-path backend/Cargo.toml --profile ci` |
| `hermes-nextest-integration` | `nextest run --manifest-path backend/Cargo.toml --profile integration --tests` |
| `hermes-llvm-cov` | `llvm-cov --manifest-path backend/Cargo.toml` |

Все перечисленные псевдонимы указывают `--manifest-path backend/Cargo.toml`.
```

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `.cargo/config.toml` | определённые псевдонимы `hermes-nextest`, `hermes-nextest-ci`, `hermes-nextest-integration`, `hermes-llvm-cov` и соответствующие им команды |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения кода, документации или ADR не видны. Других источников для сравнения не встроено.
