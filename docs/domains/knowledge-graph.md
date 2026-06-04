# Knowledge Graph Design

## Purpose

The knowledge graph represents durable relationships between people, organizations, projects, documents, messages, tasks, meetings, locations and events. It is the primary substrate for long-term memory.

## Core Entities

- Person
- Company
- Organization
- Project
- Document
- Message
- Event
- Task
- Meeting
- Location
- ChannelAccount
- Attachment
- Decision
- Promise

## Relationship Objects

Relationships are first-class records, not anonymous edges. A relationship must store:

- source entity
- target entity
- relationship type
- directionality
- confidence
- provenance
- valid time range where relevant
- created by source: user, ingestion, agent, rule or import
- review state

## Relationship Examples

- `person_works_for_company`
- `person_participated_in_meeting`
- `message_mentions_project`
- `document_related_to_project`
- `task_created_from_message`
- `decision_made_in_meeting`
- `company_related_to_project`
- `person_promised_task`

## Identity Resolution

The graph must support uncertain identity:

- multiple email addresses per person
- provider-specific usernames
- phone numbers
- aliases
- merged and split identities
- confidence-scored candidates

Automatic merges are risky. High-confidence suggestions may be staged, but user review must exist for ambiguous identities.

## Provenance

Every inferred entity or relationship must link back to evidence:

- message ID
- document version
- event ID
- extraction run
- agent run
- manual user action

Graph answers without provenance are incomplete.
