# Contacts — API Reference

Base: `/api/v2/`

## Контакты

| Метод | Путь | Описание |
|---|---|---|
| GET | `/contacts` | Список контактов (?favorites_only, ?limit) |
| GET | `/contacts/{id}` | Обогащённый профиль контакта |
| POST | `/contacts/{id}/fingerprint` | Построить communication fingerprint |
| POST | `/contacts/{id}/favorite` | Переключить избранное |
| PUT | `/contacts/{id}/notes` | Сохранить заметки |
| GET | `/contacts/search?q=...` | Поиск по имени/email |

## Identity Resolution

| Метод | Путь | Описание |
|---|---|---|
| GET | `/identity-candidates` | Список кандидатов на объединение |
| PUT | `/identity-candidates/{id}/review` | Подтвердить/отклонить |
| GET | `/contacts/{id}/identity` | Связанные identity |

## Профиль контакта (EnrichedContact)

```json
{
  "contact_id": "contact:v1:email:11:alice@ex.com",
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
