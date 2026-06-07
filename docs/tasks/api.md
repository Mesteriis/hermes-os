# Tasks — API Reference

Base: `/api/v2`

## Core

| Метод | Путь | Описание |
|---|---|---|
| GET | `/tasks` | Список (?status, ?project_id, ?source_type, ?limit) |
| POST | `/tasks` | Создать |
| GET | `/tasks/{id}` | Детали |
| PUT | `/tasks/{id}` | Обновить |
| POST | `/tasks/{id}/status` | Сменить статус |
| POST | `/tasks/{id}/archive` | Архивировать |

## Context & Evidence

| Метод | Путь |
|---|---|
| GET, POST | `/tasks/{id}/context-pack` |
| GET, POST | `/tasks/{id}/evidence` |
| GET, POST | `/tasks/{id}/relations` |
| GET, POST | `/tasks/{id}/checklist` |
| GET, POST | `/tasks/{id}/subtasks` |

## Intelligence

| Метод | Путь |
|---|---|
| POST | `/tasks/{id}/analyze` |

## Brain & Search

| Метод | Путь |
|---|---|
| POST | `/tasks/brain` |
| GET  | `/tasks/search?q=` |
| GET  | `/tasks/daily-brief` |

## Providers & External

| Метод | Путь |
|---|---|
| GET, POST | `/tasks/providers` |
| GET | `/tasks/{id}/external` |

## Rules & Templates

| Метод | Путь |
|---|---|
| GET, POST | `/tasks/rules` |
| DELETE | `/tasks/rules/{id}` |
| GET | `/tasks/templates` |

## Health & Analytics

| Метод | Путь |
|---|---|
| GET | `/tasks/watchtower` |
| GET | `/tasks/health` |
| GET | `/tasks/analytics` |

## Export

| Метод | Путь |
|---|---|
| GET | `/tasks/{id}/export?format=md\|json` |
