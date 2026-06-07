# Persons — API Reference

Base: `/api/v2/`

## Контакты

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons` | Список контактов (?favorites_only, ?limit) |
| GET | `/persons/{id}` | Обогащённый профиль контакта |
| POST | `/persons/{id}/fingerprint` | Построить communication fingerprint |
| POST | `/persons/{id}/favorite` | Переключить избранное |
| PUT | `/persons/{id}/notes` | Сохранить заметки |
| GET | `/persons/search?q=...` | Поиск по имени/email |

## Identity Resolution

| Метод | Путь | Описание |
|---|---|---|
| GET | `/identity-candidates` | Список кандидатов на объединение |
| PUT | `/identity-candidates/{id}/review` | Подтвердить/отклонить |
| GET | `/persons/{id}/identity` | Связанные identity |

## Профиль контакта (EnrichedPerson)

```json
{
  "person_id": "person:v1:email:11:alice@ex.com",
  "display_name": "Alice",
  "email_address": "alice@example.com",
  "language": "en",
  "tone": "friendly",
  "trust_score": 72,
  "avg_response_hours": 4.5,
  "preferred_channel": "email",
  "last_interaction_at": "2026-06-07T10:00:00Z",
  "interaction_count": 42,
  "frequent_topics": ["finance", "project"],
  "writing_style": "concise",
  "is_favorite": true,
  "notes": "CTO at Acme Corp",
  "linked_projects": ["hermes-hub"],
  "linked_documents": ["doc:1"]
}
```
