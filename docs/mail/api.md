# Hermes Mail — API Reference

Base: `/api/v1/communications/`

## Сообщения

| Метод | Путь | Описание |
|---|---|---|
| GET | `/messages` | Список писем (?account_id, ?workflow_state, ?channel_kind, ?limit) |
| GET | `/messages/{id}` | Детали письма с вложениями |
| PUT | `/messages/{id}/workflow-state` | Изменить workflow-состояние |
| GET | `/messages/states` | Счётчики по состояниям |
| POST | `/messages/{id}/analyze` | Запустить AI-анализ (эвристики) |
| GET | `/messages/{id}/explain` | Почему письмо важно |
| GET | `/messages/{id}/smart-cc` | Умные подсказки CC |
| POST | `/messages/{id}/pin` | Переключить pin |
| POST | `/messages/{id}/snooze` | Отложить до даты |
| POST | `/messages/{id}/mute` | Переключить mute |
| POST | `/messages/{id}/labels` | Добавить метку |
| DELETE | `/messages/{id}/labels` | Удалить метку |
| GET | `/messages/{id}/export?format=md\|eml\|json` | Экспорт письма |

## Отправка

| Метод | Путь | Описание |
|---|---|---|
| POST | `/send` | Отправить письмо |
| POST | `/messages/{id}/reply` | Ответить |
| POST | `/messages/{id}/reply-all` | Ответить всем |
| POST | `/messages/{id}/forward` | Переслать |
| POST | `/messages/{id}/forward-eml` | Переслать как EML |

## AI

| Метод | Путь | Описание |
|---|---|---|
| POST | `/messages/{id}/ai-reply` | Сгенерировать AI-ответ |
| POST | `/messages/{id}/ai-reply-variants` | Варианты ответа (языки × тоны) |
| POST | `/messages/{id}/extract-tasks` | Извлечь задачи |
| POST | `/messages/{id}/extract-notes` | Извлечь заметки |
| GET | `/messages/{id}/detect-language` | Определить язык |
| POST | `/messages/{id}/translate` | Перевести |

## Безопасность

| Метод | Путь | Описание |
|---|---|---|
| GET | `/messages/{id}/spf-dkim` | SPF/DKIM/DMARC анализ |
| GET | `/messages/{id}/signature` | Детекция подписей (S/MIME, PGP) |

## Треды

| Метод | Путь | Описание |
|---|---|---|
| GET | `/threads` | Список тредов |
| GET | `/threads/messages?account_id=&subject=` | Сообщения в треде |

## Черновики

| Метод | Путь | Описание |
|---|---|---|
| GET | `/drafts` | Список черновиков |
| POST | `/drafts` | Создать/обновить |
| GET | `/drafts/{id}` | Детали черновика |
| DELETE | `/drafts/{id}` | Удалить |

## Финансы

| Метод | Путь | Описание |
|---|---|---|
| GET | `/finance/invoices` | Список счетов |
| POST | `/finance/invoices` | Создать/обновить счёт |

## Юрдокументы

| Метод | Путь | Описание |
|---|---|---|
| GET | `/legal` | Список юрдокументов |
| POST | `/legal` | Создать/обновить |

## Сертификаты

| Метод | Путь | Описание |
|---|---|---|
| GET | `/certificates` | Список сертификатов |
| POST | `/certificates` | Добавить сертификат |
| GET | `/certificates/expiring?days=90` | Истекающие сертификаты |

## Аналитика

| Метод | Путь | Описание |
|---|---|---|
| GET | `/analytics/health` | Здоровье ящика |
| GET | `/analytics/senders` | Топ отправителей |

## Подписки

| Метод | Путь | Описание |
|---|---|---|
| GET | `/subscriptions` | Детекция рассылок |

## Поиск

| Метод | Путь | Описание |
|---|---|---|
| GET | `/search?q=...` | Полнотекстовый поиск |

## Вложения

| Метод | Путь | Описание |
|---|---|---|
| GET | `/attachments/duplicates` | Поиск дубликатов |

## Прочее

| Метод | Путь | Описание |
|---|---|---|
| GET | `/personas` | Список персон |
| POST | `/personas` | Создать персону |
| GET | `/templates/rich` | Rich-шаблоны |
| POST | `/templates/rich` | Сохранить шаблон |
| POST | `/templates/rich/render` | Отрендерить шаблон |
| GET | `/blockers` | Список архитектурных блокеров |
| POST | `/messages/{id}/imap-mark-read` | Синхронизировать read-флаг с сервером |
| POST | `/messages/{id}/imap-delete` | Удалить на сервере |
