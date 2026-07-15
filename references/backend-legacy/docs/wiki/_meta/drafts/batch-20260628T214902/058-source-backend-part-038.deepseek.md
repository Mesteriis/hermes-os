### Summary / Резюме

Страница `components/backend.md` дополняется разделом о почтовой интеграции (`backend/src/integrations/mail`). В текущем чанке представлены исходные файлы сервиса настройки учётных записей (`accounts`), Gmail‑клиента, IMAP‑клиента (чтение и запись) и транспорта для отправки писем. Страница обновляется, чтобы отразить архитектуру этих модулей, основные структуры данных, потоки OAuth/Gmail и IMAP, а также правила валидации — всё строго на основе встроенных исходников.

### Proposed pages / Предлагаемые страницы

```
`components/backend.md` — полный предлагаемый Markdown-контент (совместимость с Obsidian).
```

```markdown
# Backend компонент

## Почтовая интеграция

Модуль `backend/src/integrations/mail` объединяет:

- `accounts` – сервис настройки и токен-менеджмента для Gmail (OAuth) и IMAP‑учётных записей
- `gmail::client` – HTTP-клиент к Gmail API (получение сообщений, история, отправка)
- `gmail::client::imap` – IMAP-клиент для получения сообщений
- `imap_write` – IMAP-клиент для операций записи (пометка прочитанным, удаление)
- `outbox` – транспорт отправки писем через Gmail
- `sync` / `sync_provider` – модели и контракты синхронизации (не раскрыты в данном контексте)

---

### accounts – сервис настройки учётных записей

#### `EmailAccountSetupService`

Сервис (файл `service.rs`) инкапсулирует:

- `pool` – опциональное подключение к Postgres (`PgPool`)
- `secret_store` – опциональное хранилище ссылок на секреты (`SecretReferenceStore`)
- `provider_account_store` – опциональное хранилище провайдеров (`Arc<dyn ProviderAccountCommandPort>`)
- `provider_secret_binding_store` – опциональное хранилище привязок секретов (`Arc<dyn ProviderSecretBindingCommandPort>`)
- `vault` – `AccountSecretVault` (обёртка над `DatabaseEncryptedSecretVault` или `HostVault`)
- `http` – HTTP-клиент с таймаутом 30 секунд (создаётся через `helpers::http_client`)

Конструкторы (`service/constructors.rs`):

- `new(pool, secret_store, vault: DatabaseEncryptedSecretVault, provider_account_store, provider_secret_binding_store)`
- `new_for_vault_only(vault: DatabaseEncryptedSecretVault)` – только хранилище секретов, без привязок
- `new_with_host_vault(pool, secret_store, vault: HostVault, …)` – аналог `new`, но с `HostVault`
- `new_with_host_vault_for_token_refresh(pool, secret_store, vault: HostVault)` – минимальный набор для обновления токена

#### Gmail OAuth – поток настройки

1. **Старт** – `start_gmail_oauth(GmailOAuthSetupRequest) -> GmailOAuthPendingGrant`  
   (`gmail_start.rs`)  
   - Валидирует запрос (`GmailOAuthSetupRequest::validate`).  
   - Генерирует `setup_id`, `state`, `code_verifier` (случайные токены, `random_url_token`).  
   - Вычисляет `code_challenge` = `pkce_challenge(code_verifier)` (SHA-256, base64url).  
   - Формирует URL авторизации с параметрами: `response_type=code`, `client_id`, `redirect_uri`, `scope` (сериализованные через пробел), `state`, `code_challenge`, `code_challenge_method=S256`, `access_type=offline`, `prompt=consent`.  
   - Возвращает `GmailOAuthPendingGrant`, содержащий `authorization_url`.

2. **Завершение** – `complete_gmail_oauth(pending, authorization_code) -> EmailAccountSetupResult`  
   (`gmail_complete.rs`)  
   - Обменивает `authorization_code` через `exchange_authorization_code` (HTTP POST к `token_endpoint`).  
   - Из ответа извлекает `refresh_token`; если отсутствует – ошибка `MissingProviderField{ refresh_token }`.  
   - Время истечения: `expires_at = expires_at(expires_in)` (входной `expires_in` уменьшается на 60 сек, но не меньше 60 сек).  
   - Собирает `GmailOAuthTokenBundle` (поля: `token_url`, `client_id`, `client_secret`, `access_token`, `refresh_token`, `expires_at`, `token_type`, `scope`).  
   - Сохраняет ссылку на секрет (`upsert_secret_reference`) с `SecretKind::OauthToken` и метаданными (см. ниже).  
   - Сохраняет сам токен в хранилище (`AccountSecretVault::store_secret`) с контекстом `entry_kind="provider_credential"`, `purpose=OauthToken`, `secret_kind=OauthToken`, метаданными.  
   - Создаёт или обновляет запись провайдера (`NewProviderAccount`) с `EmailProviderKind::Gmail`.  
   - Привязывает секрет (`NewProviderAccountSecretBinding`) с `purpose=OauthToken`.  
   - Возвращает `EmailAccountSetupResult` с `secret_ref = "secret:provider-account:{account_id}:oauth_token"`.

3. **Обновление токена** – `refresh_gmail_access_token(secret_ref)`  
   (`gmail_refresh.rs`)  
   - Читает сохранённый `GmailOAuthTokenBundle`.  
   - Если `bundle.expires_at > now + 60s` – возвращает текущий `access_token` без обновления.  
   - Иначе отправляет `refresh_token` на `bundle.token_url` (`refresh_token` flow).  
   - Обновляет поля бандла (access_token, refresh_token, expires_at, token_type, scope) и перезаписывает секрет.

**Метаданные** (`gmail_payloads.rs`):
- Конфиг аккаунта (`gmail_account_config`): JSON с полями `auth="oauth"`, `api="gmail"`, `oauth_client_id`, `requested_scopes`, `gmail_send_enabled` (флаг наличия `GOOGLE_GMAIL_SEND_SCOPE`), `connected_services=["mail","calendar","contacts"]`, `history_stream_id="gmail:history"`.
- Метаданные секрета (`gmail_secret_metadata`): `provider="gmail"`, `account_id`, `display_name`, `external_account_id`, `connected_services`, `provider_account_config`.

#### IMAP – настройка учётной записи

`setup_imap_account(ImapAccountSetupRequest) -> EmailAccountSetupResult` (`imap.rs`):

- Валидация `ImapAccountSetupRequest::validate`:
  - Обязательные поля: `account_id`, `display_name`, `external_account_id`, `host`, `mailbox`, `username`, `password`.
  - `port > 0`, `smtp_port > 0`.
  - `provider_kind` должен быть `Icloud` или `Imap` (Gmail запрещён).
  - `secret_kind` допустим только `AppPassword` или `Password`.
- SMTP-конфиг вычисляется методом `smtp_config`:
  - `host` – из поля `smtp_host`, для iCloud по умолчанию `"smtp.mail.me.com"`, иначе `host`.
  - `port` – `smtp_port` либо `587`.
  - `tls` – `smtp_tls` либо `true`.
  - `starttls` – `smtp_starttls` либо `true`.
  - `username` – из `smtp_username`, иначе `external_account_id`.
- Создаются ссылки на секреты: `imap_secret_ref = "secret:provider-account:{account_id}:imap_password"`, `smtp_secret_ref = "secret:provider-account:{account_id}:smtp_password"`.
- Пароль сохраняется дважды (IMAP и SMTP) с контекстом `purpose=ImapPassword`/`SmtpPassword`.
- Создаётся провайдер-аккаунт с `provider_kind` из запроса и конфигом `imap_account_config`.
- Привязываются оба секрета (`ImapPassword`, `SmtpPassword`).

**Метаданные** (`imap_payloads.rs`):
- Конфиг аккаунта (`imap_account_config`): JSON с `host`, `port`, `tls`, `mailbox`, `username`, а также SMTP-поля.
- Если `email_provider_connected_services` возвращает сервисы (для `Gmail`/`Icloud`), они вшиваются в `connected_services`.

#### Вспомогательные функции (`helpers.rs`)

- `http_client()` – reqwest-client с таймаутом 30 секунд.
- `expires_at(expires_in)` – `Utc::now() + max(expires_in.unwrap_or(3600) - 60, 60)` секунд.
- Ссылки на секреты: `oauth_secret_ref`, `imap_secret_ref`, `smtp_secret_ref`.
- `email_provider_connected_services` – для `Gmail`/`Icloud` возвращает `["mail","calendar","contacts"]`, для остальных – `None`.
- `vault_secret_reference` – строит `SecretReference` с текущим временем и меткой `"encrypted vault secret"`.
- `random_url_token()` – 32 случайных байта, base64url без padding.
- `pkce_challenge(code_verifier)` – SHA-256 от верификатора, base64url.

#### Vault (`vault.rs`)

`AccountSecretVault` – перечисление:
- `Database(DatabaseEncryptedSecretVault)`
- `Host(HostVault)`

Методы:
- `store_kind()` – возвращает `DatabaseEncryptedVault` или `HostVault`.
- `store_secret(secret_ref, value, SecretWriteContext)` – сохраняет строку; контекст содержит `entry_kind`, `account_id`, `purpose`, `secret_kind`, `label`, `metadata`.
- `secret_reference(secret_ref, secret_kind)` – строит `SecretReference` через `vault_secret_reference`.
- Реализует trait `SecretResolver` – делегирует разрешение вложенному vault.

---

### Gmail API клиент

Модуль `backend/src/integrations/mail/gmail/client`.

#### `GmailApiClient` (`gmail_api.rs`)

Конструктор `new(base_url)` – создаёт HTTP-клиент с таймаутом 30 секунд, `user_id` по умолчанию `"me"`. URL обрезается через `trim_base_url`. Методы:

- `fetch_raw_messages(access_token, GmailFetchOptions)` – получает список сообщений и их полное содержимое (формат `raw`).
  - Опции: `max_results` (1..500), `query`, `page_token`, `label_ids`, `include_spam_trash`.
  - Для каждого сообщения вычисляется `source_fingerprint` = `sha256(["gmail".as_bytes(), provider_record_id, raw])`.
  - Чекпоинт (JSON) включает `history_id` из максимального идентификатора истории среди всех сообщений и `next_page_token`.
- `fetch_history_raw_messages(access_token, GmailHistoryFetchOptions)` – инкрементальная выборка через `history.list`.
  - Опции: `start_history_id`, `max_results` (1..500), `page_token`.
  - По каждой записи истории извлекает идентификаторы добавленных сообщений, затем через `fetch_raw_message` получает их содержимое (с дедупликацией по id, лимитируется до `max_results`).
  - Чекпоинт сохраняет `history_id`, `next_page_token`, `page_kind="history"`, `start_history_id`.
- `send_message(access_token, OutgoingEmail)` – кодирует RFC‑2822 в base64url и отправляет через `POST /gmail/v1/users/{user_id}/messages/send`. Требует хотя бы одного получателя.

Вспомогательные функции (`helpers.rs`):
- `trim_base_url` – удаляет завершающий `/`.
- `validate_non_empty` – проверка непустого строкового поля.
- `parse_gmail_internal_date` – парсит миллисекунды эпохи в `DateTime<Utc>`.
- `select_latest_history_id` – выбирает больший `history_id` (сравнение как u64).
- `gmail_message_list_checkpoint` / `gmail_history_checkpoint` / `imap_checkpoint` – построение чекпоинтов в виде JSON.
- `sha256_fingerprint` – хеш SHA-256 от конкатенации нескольких срезов, формат `"sha256:hex"`.

**Модели** (`models.rs`):
- `GmailListResponse` (messages, next_page_token)
- `GmailListedMessage` (id, thread_id)
- `GmailRawMessage` (id, thread_id, label_ids, history_id, internal_date, raw)
- `GmailSendResponse` (id)
- `GmailHistoryResponse` (history, history_id, next_page_token)
- `GmailHistoryItem` (messages_added – vec of `GmailHistoryMessageAdded`)
- `GmailHistoryMessageAdded` (message: `GmailHistoryMessage`)
- `GmailHistoryMessage` (id)

**Ошибки** (`errors.rs`): `EmailProviderNetworkError` – варианты: `InvalidProviderRequest`, `InvalidProviderResponse`, `MissingProviderField`, `UnexpectedProviderResponse`, `ProviderTimeout`, `Http`, `Io`, `Tls`, `Imap`.

**Опции** (`options.rs`):
- `GmailFetchOptions` – `max_results` (1..500), `query`, `page_token`, `label_ids`, `include_spam_trash`.
- `GmailHistoryFetchOptions` – `start_history_id`, `max_results` (1..500), `page_token`.
- `ImapFetchOptions` – `provider_kind`, `host`, `port`, `tls`, `mailbox`, `username`, `last_seen_uid`, `max_messages`, `latest_messages` (флаг).

---

### IMAP клиент (чтение) – `imap.rs`

`ImapNetworkClient` – stateless (derive `Default`). Единственный публичный метод:

- `fetch_raw_messages(password, ImapFetchOptions) -> EmailSyncBatch`
  - Устанавливает TCP-соединение; при `tls=true` оборачивает в TLS через `async_native_tls`.
  - Авторизуется, вызывает `examine(mailbox)`, читает `uid_validity`.
  - Вычисляет нижнюю границу UID: `next_imap_uid_floor(last_seen_uid)`. Если `last_seen_uid == None`, старт с UID 1.
  - Поиск UID: `UID {first_uid}:*`, затем фильтрация через `retain_uids_from_floor` и выборка через `select_uids_for_fetch`.
  - Выборка пачками по 10 UID с таймаутом 60 секунд на пачку. Для каждого сообщения:
    - UID, тело (RFC‑822), опционально `internal_date`.
    - `source_fingerprint` = sha256(["imap", uid, body]).
    - `payload` – JSON с `provider`, `transport`, `mailbox`, `uid`, `uid_validity`, `raw_rfc822_base64` (base64 стандартный), `rfc822_size`.
  - Чекпоинт: `imap_checkpoint(mailbox, uid_validity, latest_uid)`.
  - В конце сессия `logout`.

Вспомогательные:
- `next_imap_uid_floor` – возвращает `last_seen_uid + 1` или `None` при переполнении u32.
- `uid_set` – сериализация UID через запятую.
- `retain_uids_from_floor` – фильтрует UID >= нижняя граница.
- `select_uids_for_fetch` – сортирует, при `latest_messages=true` берёт последние `max_messages`, иначе первые.
- `imap_checkpoint` – JSON с `provider="imap"`, `mailbox`, `uid_validity`, `last_seen_uid`.

---

### IMAP запись – `imap_write.rs`

`ImapWriteClient` – stateless (derive `Default`). Операции принимают `ImapWriteConfig`:
- `host`, `port`, `tls`, `username`, `password: &ResolvedSecret`, `mailbox`.

Методы:
- `mark_seen(uids)` – `UID STORE {uid_set} +FLAGS (\Seen)`.
- `delete_messages(uids)` – `UID STORE {uid_set} +FLAGS (\Deleted)`, затем `EXPUNGE`.

Вспомогательная `with_imap_session` поднимает TCP/TLS, логинится, вызывает переданное замыкание с `async_imap::Session`. Ошибки `ImapWriteError` покрывают `Io`, `Tls`, `Imap`.

---

### Исходящий транспорт (outbox) – `outbox.rs`

`LiveGmailOutboxTransport` реализует трейт `GmailOutboxTransport`:
- Конструктор `new(pool: PgPool, vault: HostVault)` – создаёт `SecretReferenceStore` и транспорт.
- Метод `send(request: GmailOutboxSendRequest)`:
  1. Создаёт `EmailAccountSetupService::new_with_host_vault_for_token_refresh` для обновления токена.
  2. Вызывает `refresh_gmail_access_token(request.oauth_secret_ref)`.
  3. Отправляет письмо через `GmailApiClient::new(request.api_base_url).user_id("me").send_message(access_token, request.email)`.

---

### Ссылки на секреты (кратко)

Все ссылки на секреты строятся по шаблонам:
- Gmail OAuth: `secret:provider-account:{account_id}:oauth_token`
- IMAP: `secret:provider-account:{account_id}:imap_password`
- SMTP: `secret:provider-account:{account_id}:smtp_password`

Ссылки сопровождаются `SecretReference` с меткой `"encrypted vault secret"` и соответствующим `SecretKind`.
```

### Source coverage / Покрытие источников

| Файл | Факты, покрытые в предлагаемой странице |
|---|---|
| `backend/src/integrations/mail/accounts/helpers.rs` | `http_client`, `expires_at`, `oauth_secret_ref`, `imap_secret_ref`, `smtp_secret_ref`, `email_provider_connected_services`, `vault_secret_reference`, `random_url_token`, `pkce_challenge`. |
| `backend/src/integrations/mail/accounts/models.rs` | `GmailOAuthSetupRequest` (поля, new, builder-методы, `validate`), `GmailOAuthPendingGrant`, `ImapAccountSetupRequest` (поля, new, smtp-конфиг, `validate`), `ImapAccountSmtpConfig`, `EmailAccountSetupResult`, `GmailOAuthTokenBundle`, `OAuthTokenResponse`. |
| `backend/src/integrations/mail/accounts/service.rs` | Структура `EmailAccountSetupService` (поля, видимость). |
| `backend/src/integrations/mail/accounts/service/constructors.rs` | Четыре конструктора: `new`, `new_for_vault_only`, `new_with_host_vault`, `new_with_host_vault_for_token_refresh`. |
| `backend/src/integrations/mail/accounts/service/gmail_complete.rs` | Поток `complete_gmail_oauth`: обмен кода, обязательность `refresh_token`, сохранение токена, создание `NewProviderAccount` и `NewProviderAccountSecretBinding`. |
| `backend/src/integrations/mail/accounts/service/gmail_payloads.rs` | `gmail_account_config`, `gmail_secret_metadata`, `gmail_send_scope_requested`. |
| `backend/src/integrations/mail/accounts/service/gmail_refresh.rs` | `refresh_gmail_access_token`: проверка `expires_at > now+60`, обновление токена и перезапись секрета. |
| `backend/src/integrations/mail/accounts/service/gmail_start.rs` | `start_gmail_oauth`: генерация PKCE, параметры URL, `GmailOAuthPendingGrant`. |
| `backend/src/integrations/mail/accounts/service/imap.rs` | `setup_imap_account`: валидация, сохранение IMAP и SMTP паролей, провайдер-аккаунт, привязки. |
| `backend/src/integrations/mail/accounts/service/imap_payloads.rs` | `imap_account_config`, `imap_secret_metadata`, использование `email_provider_connected_services`. |
| `backend/src/integrations/mail/accounts/service/stores.rs` | Методы доступа к опциональным store-полям (`pool`, `secret_store`, `provider_account_store`, `provider_secret_binding_store`). |
| `backend/src/integrations/mail/accounts/service/token_http.rs` | `exchange_authorization_code`, `refresh_token` – HTTP POST формы с возможным `client_secret`. |
| `backend/src/integrations/mail/accounts/validation.rs` | `validate_non_empty` (версия модуля accounts). |
| `backend/src/integrations/mail/accounts/vault.rs` | `AccountSecretVault` (Database/Host), `store_kind`, `store_secret`, `secret_reference`, реализация `SecretResolver`. `SecretWriteContext`. |
| `backend/src/integrations/mail/gmail/client.rs` | Экспорты модуля (`GmailApiClient`, `ImapNetworkClient`, `EmailProviderNetworkError`, опции). |
| `backend/src/integrations/mail/gmail/client/errors.rs` | `EmailProviderNetworkError` – варианты ошибок. |
| `backend/src/integrations/mail/gmail/client/gmail_api.rs` | `GmailApiClient` – конструктор, `fetch_raw_messages`, `fetch_history_raw_messages`, `send_message`, `fetch_raw_message`. Формат запросов, параметры. |
| `backend/src/integrations/mail/gmail/client/helpers.rs` | `trim_base_url`, `validate_non_empty`, `parse_gmail_internal_date`, `select_latest_history_id`, чекпоинты, `sha256_fingerprint`, `hex_lower`. |
| `backend/src/integrations/mail/gmail/client/imap.rs` | `ImapNetworkClient::fetch_raw_messages` – TCP/TLS, схема UID-поиска, чанкированная выборка, чекпоинты. |
| `backend/src/integrations/mail/gmail/client/models.rs` | Структуры ответов Gmail JSON (list, raw, history). |
| `backend/src/integrations/mail/gmail/client/options.rs` | `GmailFetchOptions`, `GmailHistoryFetchOptions`, `ImapFetchOptions` – поля и валидация. |
| `backend/src/integrations/mail/gmail/mod.rs` | Модуль `gmail::client` (корневой re-export). |
| `backend/src/integrations/mail/imap_write.rs` | `ImapWriteClient` – `mark_seen`, `delete_messages`, `ImapWriteConfig`, `with_imap_session`. |
| `backend/src/integrations/mail/mod.rs` | Состав модуля `mail`. |
| `backend/src/integrations/mail/outbox.rs` | `LiveGmailOutboxTransport` – конструктор, метод `send`, обновление токена через `refresh_gmail_access_token`, отправка через `GmailApiClient`. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения между кодом, документацией и ADR не видны. Все описанные элементы непосредственно присутствуют во встроенных исходных файлах и не противоречат друг другу.
