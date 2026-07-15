# Calendar And Events Domain

Calendar and Events represent scheduled, observed and attended happenings.

This domain owns event records. It does not own the global Timeline Engine.

## Responsibilities

The Calendar/Events domain owns:

- calendar accounts and provider event identity;
- meetings;
- calls when represented as events;
- attendees and participation records;
- schedules and recurrence metadata;
- reminders and scheduling rules;
- event evidence and links.

The domain does not own:

- global timeline assembly;
- task lifecycle;
- Persona identity;
- meeting summary truth beyond cited evidence;
- communication message source records.

## Event Types

Events include:

- calendar meetings;
- calls;
- deadlines when represented as scheduled events;
- reminders;
- imported provider events;
- future observed lifecycle events that require scheduling semantics.

Communications may happen inside events. Events may generate communications,
documents, decisions, obligations and tasks.

## Meeting Memory

Meeting memory is built from:

- calendar source evidence;
- attendees;
- linked communications;
- notes or documents;
- decisions;
- obligations;
- tasks;
- follow-up communications.

The meeting context view is derived and must cite source records.

## Current Implementation Evidence

Current backend implementation includes:

- `backend/src/domains/calendar/*`;
- `backend/src/platform/calls/*`;
- calendar migrations `0044` through `0047`;
- route groups for calendar and calls;
- Calendar frontend page.

## Migration Plan

1. Keep Calendar/Events distinct from the global Timeline Engine.
2. Treat calls and meetings as event-capable communication contexts.
3. Link decisions, obligations and tasks to event evidence.
4. Avoid building separate timeline logic inside Calendar docs; use Timeline
   Engine language.
5. Add event-to-communication relationship semantics to future graph plans.
