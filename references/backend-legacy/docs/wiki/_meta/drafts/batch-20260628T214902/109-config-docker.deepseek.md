### Summary / Резюме

Добавить в `operations/configuration.md` раздел о конфигурации Docker Compose проекта `hermes-hub-dev`. Исходный файл `docker/docker-compose.yml` определяет три сервиса: `postgres`, `nats`, `whatsapp-business-cloud-edge-proxy`. Документируются образы, переменные окружения, проброс портов, тома, healthcheck‑и и активация через профили. Всё содержимое основано исключительно на предоставленном `docker-compose.yml`; что-либо за его пределами не утверждается.

### Proposed pages / Предлагаемые страницы

`operations/configuration.md`

```markdown
# Конфигурация Docker Compose

В этом документе описаны сервисы и переменные окружения, управляемые через
`docker/docker-compose.yml` проекта `hermes-hub-dev`.

Все обязательные переменные окружения должны быть заданы в файле `docker/.env`.
В композ-файле используется синтаксис `${VAR:?error}` для обязательных
переменных — при их отсутствии `docker compose` выдаст ошибку.

## Сервис `postgres`

- **Образ**: `pgvector/pgvector:0.8.2-pg16`
- **Политика перезапуска**: `unless-stopped`

### Переменные окружения (обязательные)

| Переменная в контейнере | Значение из `.env`      |
|--------------------------|-------------------------|
| `POSTGRES_DB`            | `HERMES_POSTGRES_DB`    |
| `POSTGRES_USER`          | `HERMES_POSTGRES_USER`  |
| `POSTGRES_PASSWORD`      | `HERMES_POSTGRES_PASSWORD` |

### Порты

- Хост: `${HERMES_POSTGRES_BIND:-127.0.0.1}:${HERMES_POSTGRES_PORT:-5432}`
- Контейнер: `5432`

### Том

- `./data/postgres` → `/var/lib/postgresql/data` (bind mount)

### Healthcheck

- Команда: `pg_isready -U "${POSTGRES_USER}" -d "${POSTGRES_DB}"`
- Интервал: `10s`, таймаут: `5s`, ретраи: `5`, start_period: `10s`

## Сервис `nats`

- **Образ**: `nats:2.11-alpine`
- **Политика перезапуска**: `unless-stopped`

### Аргументы команды

- `-js` — включение JetStream
- `-sd /data` — директория хранения данных JetStream
- `-m 8222` — порт HTTP‑мониторинга

### Порты

- Клиентский: хост `${HERMES_NATS_BIND:-127.0.0.1}:${HERMES_NATS_PORT:-4222}` → контейнер `4222`
- Мониторинг: хост `${HERMES_NATS_MONITOR_BIND:-127.0.0.1}:${HERMES_NATS_MONITOR_PORT:-8222}` → контейнер `8222`

### Том

- `./data/nats` → `/data` (bind mount)

### Healthcheck

- Команда: `wget -q -O - http://127.0.0.1:8222/healthz`
- Интервал: `10s`, таймаут: `5s`, ретраи: `5`, start_period: `5s`

## Сервис `whatsapp-business-cloud-edge-proxy`

- **Профиль**: `whatsapp-business-cloud-edge` — сервис запускается только при
  явном указании этого профиля (например, `docker compose --profile whatsapp-business-cloud-edge up`)
- **Сборка**: контекст `..`, Dockerfile `docker/Dockerfile`, target
  `whatsapp-business-cloud-edge-proxy`
- **Политика перезапуска**: `unless-stopped`

### Переменные окружения

| Переменная в контейнере                                         | Источник / значение по умолчанию                                                                  |
|-----------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR`                 | Зафиксировано: `0.0.0.0:8787`                                                                    |
| `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL`           | `${HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL:-http://host.docker.internal:8080}`       |
| `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID`                | `${HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID:-}` (по умолчанию пусто)                       |
| `HERMES_LOCAL_API_SECRET`                                       | **Обязательная**. `${HERMES_LOCAL_API_SECRET}` из `docker/.env`                                  |
| `HERMES_LOG_FORMAT`                                             | `${HERMES_LOG_FORMAT:-json}`                                                                      |
| `RUST_LOG`                                                      | `${HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_RUST_LOG:-info}`                                           |

### Порты

- Хост: `${HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND:-127.0.0.1}:${HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT:-8787}`
- Контейнер: `8787`

### Extra hosts

- `host.docker.internal` → `host-gateway`

### Healthcheck

- Команда: `curl -fsS http://127.0.0.1:8787/healthz >/dev/null`
- Интервал: `10s`, таймаут: `5s`, ретраи: `5`, start_period: `5s`
```

### Source coverage / Покрытие источников

*   `docker/docker-compose.yml`:
    *   Имя проекта `hermes-hub-dev`.
    *   Сервис `postgres`: образ `pgvector/pgvector:0.8.2-pg16`, restart `unless-stopped`, обязательные переменные `POSTGRES_DB`/`POSTGRES_USER`/`POSTGRES_PASSWORD`, маппинг портов с дефолтным биндом `127.0.0.1` и портом `5432`, bind‑том `./data/postgres` → `/var/lib/postgresql/data`, healthcheck `pg_isready`.
    *   Сервис `nats`: образ `nats:2.11-alpine`, restart `unless-stopped`, аргументы `-js -sd /data -m 8222`, маппинг клиентского и мониторингового портов, bind‑том `./data/nats` → `/data`, healthcheck `wget …/healthz`.
    *   Сервис `whatsapp-business-cloud-edge-proxy`: профиль `whatsapp-business-cloud-edge`, сборка из корня через `docker/Dockerfile` c target `whatsapp-business-cloud-edge-proxy`, restart `unless-stopped`, полный список переменных окружения с фиксированным `BIND_ADDR=0.0.0.0:8787`, дефолтный `HERMES_BASE_URL`, дефолтный пустой `ACCOUNT_ID`, обязательный `HERMES_LOCAL_API_SECRET`, дефолтный `LOG_FORMAT=json`, `RUST_LOG` из `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_RUST_LOG` с дефолтом `info`, маппинг порта `8787`, extra‑host `host.docker.internal:host-gateway`, healthcheck `curl …/healthz`.

### Drift candidates / Кандидаты на drift

- **Жёстко зафиксированный `BIND_ADDR` сервиса `whatsapp-business-cloud-edge-proxy`**: переменная `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR` всегда равна `0.0.0.0:8787`, в то время как порт на хосте можно изменить через `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT`. Если эта переменная задана и отличается от `8787`, сервис всё равно будет слушать `8787`, что приведёт к несоответствию между тем, что слушает контейнер, и тем, что пробрасывается на хост. Аналогично, healthcheck обращается к `127.0.0.1:8787`, что может перестать соответствовать ожидаемому порту. Явного описания этого ограничения в предоставленном контексте нет — страница вики должна отразить, что изменение `HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT` без переопределения `BIND_ADDR` приведёт к расхождению.
