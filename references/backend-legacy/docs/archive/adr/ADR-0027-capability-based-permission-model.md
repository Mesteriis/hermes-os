# ADR-0027 Capability Based Permission Model

Status: Proposed

## Context

Agents and plugins may read private data or perform side effects such as sending messages, exporting data or deleting records.

## Decision

Use a capability-based permission model for agents, plugins and external actions.

## Consequences

- Permissions can be scoped and audited.
- High-risk actions can require explicit confirmation.
- Capability manifests become part of plugin and tool contracts.
- Policy evaluation must be centralized enough to be reliable.
