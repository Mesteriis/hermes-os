# UI Architecture

## UI Goals

- desktop-first productivity;
- keyboard-first operation;
- dense but readable information;
- contextual AI actions throughout the product;
- responsive desktop layout;
- modern interaction quality.

The UI is not a collection of app clones. It is a Personal Operating System
surface over the Personal Memory System.

## Primary Surfaces

- communication context;
- command palette;
- Persona workspace;
- Organization workspace;
- Project workspace;
- document viewer;
- Task and Obligation view;
- memory search;
- graph explorer;
- agent activity drawer;
- settings and permissions.

## Navigation Model

The UI should support:

- global command palette;
- quick switcher for Personas, Organizations, Projects, Documents and Tasks;
- keyboard shortcuts;
- split panes;
- contextual sidebars;
- breadcrumb history;
- saved views.

## State Model

Frontend state should distinguish:

- server-backed canonical state;
- optimistic command state;
- local view state;
- search filters;
- agent workflow state;
- draft state.

Durable owner changes must pass through backend commands.

## AI Interaction Model

AI actions should be embedded where context exists:

- summarize communication context;
- extract Task or Obligation candidates;
- explain why entities are linked;
- find related documents;
- draft reply;
- analyze changes;
- prepare meeting context.

The UI must show source references and distinguish generated text from source
content.
