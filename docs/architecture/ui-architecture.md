# UI Architecture

## UI Goals

- desktop-first productivity
- keyboard-first operation
- dense but readable information
- contextual AI actions throughout the product
- responsive layout
- modern interaction quality

## Primary Surfaces

- unified inbox and communication timeline
- command palette
- person profile
- project workspace
- document viewer
- task view
- memory search
- graph explorer
- agent activity drawer
- settings and permissions

## Navigation Model

The UI should support:

- global command palette
- quick switcher for persons, projects, documents and tasks
- keyboard shortcuts
- split panes
- contextual sidebars
- breadcrumb history
- saved views

## State Model

Frontend state should distinguish:

- server-backed canonical state
- optimistic command state
- local view state
- search filters
- agent workflow state
- draft state

Durable user changes must pass through backend commands.

## AI Interaction Model

AI actions should be embedded where context exists:

- summarize thread
- extract task
- explain why linked
- find related documents
- draft reply
- analyze changes
- prepare meeting context

The UI must show source references and distinguish generated text from source content.
