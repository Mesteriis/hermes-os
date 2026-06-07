# Organizations — API Reference

Base: `/api/v1/`

## Core

| Метод | Путь | Описание |
|---|---|---|
| GET | `/organizations` | Список (?org_type, ?limit) |
| POST | `/organizations` | Создать |
| GET | `/organizations/{id}` | Профиль |
| PUT | `/organizations/{id}` | Обновить |
| GET | `/organizations/search?q=` | Поиск |
| POST | `/organizations/{id}/archive` | Архивировать |

## Identities & Aliases

| Метод | Путь |
|---|---|
| GET, POST | `/organizations/{id}/identities` |
| GET, POST | `/organizations/{id}/aliases` |
| GET | `/organizations/{id}/domains` |

## Departments & Contacts

| Метод | Путь |
|---|---|
| GET, POST | `/organizations/{id}/departments` |
| GET, POST | `/organizations/{id}/contacts` |
| GET | `/organizations/{id}/related` |

## Timeline & Templates

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/timeline` |
| GET | `/organizations/{id}/templates` |

## Portals, Procedures, Playbooks

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/portals` |
| GET | `/organizations/{id}/procedures` |
| GET | `/organizations/{id}/playbooks` |

## Finance

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/financial` |
| GET | `/organizations/{id}/contracts` |
| GET | `/organizations/{id}/compliance` |
| GET | `/organizations/{id}/services` |
| GET | `/organizations/{id}/products` |

## Enrichment

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/enrichment` |
| POST | `/organizations/{id}/enrichment/{rid}/apply` |

## Risks & Health

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/risks` |
| GET | `/organizations/{id}/health` |
| POST | `/organizations/{id}/watchlist` |

## Investigator

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/dossier` |
| GET | `/organizations/{id}/brief` |
| GET | `/organizations/{id}/context-pack` |
