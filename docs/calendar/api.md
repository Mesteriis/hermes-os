# Calendar — API Reference

Base: `/api/v1/calendar`

## Accounts

| Метод | Путь | Описание |
|---|---|---|
| GET | `/accounts` | Список (?provider) |
| POST | `/accounts` | Создать |
| GET | `/accounts/{id}` | Профиль |
| PUT | `/accounts/{id}` | Обновить |
| DELETE | `/accounts/{id}` | Удалить |
| GET | `/accounts/{id}/sources` | Календари аккаунта |
| POST | `/accounts/{id}/sources` | Добавить календарь |
| POST | `/accounts/{id}/sync` | Триггер синхронизации |

## Events

| Метод | Путь | Описание |
|---|---|---|
| GET | `/events` | Список (?account_id, ?source_id, ?from, ?to, ?status, ?event_type, ?limit) |
| POST | `/events` | Создать |
| GET | `/events/{id}` | Детали |
| PUT | `/events/{id}` | Обновить |
| DELETE | `/events/{id}` | Удалить |
| POST | `/events/{id}/reschedule` | Перенести |
| POST | `/events/{id}/cancel` | Отменить |

## Participants & Relations

| Метод | Путь |
|---|---|
| GET, POST | `/events/{id}/participants` |
| GET, POST | `/events/{id}/relations` |

## Context & Preparation

| Метод | Путь |
|---|---|
| GET, POST | `/events/{id}/context-pack` |
| GET, POST | `/events/{id}/agenda` |
| GET, POST | `/events/{id}/checklist` |

## Intelligence

| Метод | Путь |
|---|---|
| POST | `/events/{id}/classify` |
| POST | `/events/{id}/analyze` |
| GET | `/events/{id}/risks` |
| GET | `/events/{id}/brief` |
| POST | `/events/{id}/generate-agenda` |

## Meetings

| Метод | Путь |
|---|---|
| GET, POST | `/events/{id}/notes` |
| GET, POST | `/events/{id}/outcomes` |
| GET, POST | `/events/{id}/recording` |
| GET | `/events/{id}/transcript` |
| POST | `/events/{id}/follow-up` |
| GET | `/events/{id}/follow-up-status` |

## Deadlines & Focus

| Метод | Путь |
|---|---|
| GET, POST | `/deadlines` |
| GET, POST | `/focus-blocks` |
| POST | `/smart-schedule` |

## Health & Analytics

| Метод | Путь |
|---|---|
| GET | `/watchtower` |
| GET | `/health` |
| GET | `/weekly-brief` |
| GET | `/analytics` |

## Brain & Search

| Метод | Путь |
|---|---|
| POST | `/brain` |
| GET | `/search?q=` |

## Rules

| Метод | Путь |
|---|---|
| GET, POST | `/rules` |
| PUT, DELETE | `/rules/{id}` |

## Import/Export

| Метод | Путь |
|---|---|
| POST | `/import` |
| GET | `/events/{id}/export?format=ics\|md\|json` |
