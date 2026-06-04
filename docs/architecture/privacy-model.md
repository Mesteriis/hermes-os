# Privacy Model

## Privacy Goals

- user owns local data
- cloud dependencies are optional
- generated insights remain traceable
- deletion and export are first-class capabilities
- private source content is not used for fine-tuning

## Data Classes

| Class | Examples | Default handling |
| --- | --- | --- |
| Raw source data | provider message, attachment, imported file | preserve with provenance |
| Canonical entities | Person, Project, Task, Document | local relational storage |
| Derived data | summaries, extracted entities, embeddings | local and rebuildable where possible |
| Sensitive data | secrets, tokens, credentials | encrypted secret store |
| Audit data | tool calls, permission events | local append-only audit trail |

## AI Privacy Rules

- No fine-tuning on private user data.
- Prefer local models through Ollama.
- Remote model use, if added, must be opt-in per workflow or policy.
- Prompts to external services must be logged as privacy-relevant events without storing secrets.
- Summaries must link to sources and confidence.

## Deletion Model

Deletion must handle:

- raw source record deletion
- canonical entity deletion or tombstone
- derived artifact invalidation
- graph edge removal
- search and vector index removal
- backup retention caveats

Destructive deletion should be explicit and audited. For many cases, archival or tombstone semantics are safer than hard delete.

## Export Model

The user must be able to export:

- raw imported records where legal and technically possible
- normalized messages
- contacts
- documents
- tasks
- graph edges
- event history

Exports must be structured, documented and independent from a specific LLM provider.
