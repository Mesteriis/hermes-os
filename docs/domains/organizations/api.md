# Organizations — API Reference

This file documents current compatibility routes. Canonical Organization,
Relationship and Persona terminology is defined in `../../foundation/glossary.md`.

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

## Departments & Persona Links

| Метод | Путь | Описание |
|---|---|---|
| GET, POST | `/organizations/{id}/departments` | |
| GET, POST | `/organizations/{id}/persona-links` | Organization-Persona links |
| GET | `/organizations/{id}/related` | |

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

## Risk & Attention

| Метод | Путь | Описание |
|---|---|---|
| GET | `/organizations/{id}/risks` | |
| GET | `/organizations/{id}/health` | Compatibility route for an attention/risk read model |
| POST | `/organizations/{id}/watchlist` | Compatibility route for attention/read-model state |

## Dossier & Context

| Метод | Путь |
|---|---|
| GET | `/organizations/{id}/dossier` |
| GET | `/organizations/{id}/brief` |
| GET | `/organizations/{id}/context-pack` |
