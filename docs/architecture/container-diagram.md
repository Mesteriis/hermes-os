# Container Diagram

## Containers

```mermaid
flowchart TB
    subgraph Desktop["Desktop application"]
        Tauri["Tauri shell"]
        Svelte["SvelteKit UI"]
    end

    subgraph Backend["Rust backend"]
        API["API server"]
        Commands["Command handlers"]
        Queries["Query handlers"]
        Ingestion["Ingestion workers"]
        Projectors["Projection workers"]
        AgentRuntime["Agent runtime"]
        PluginHost["Plugin host"]
    end

    subgraph Storage["Local storage"]
        Postgres["PostgreSQL"]
        EventLog["Event tables"]
        Graph["Graph tables"]
        ObjectStore["Document storage"]
        Tantivy["Tantivy indexes"]
        Vector["Vector indexes"]
    end

    subgraph AI["Local AI"]
        Ollama["Ollama"]
        Embeddings["Embedding models"]
        OCR["OCR engine"]
    end

    Tauri --> Svelte
    Svelte --> API
    API --> Commands
    API --> Queries
    Commands --> EventLog
    Ingestion --> EventLog
    EventLog --> Projectors
    Projectors --> Postgres
    Projectors --> Graph
    Projectors --> ObjectStore
    Projectors --> Tantivy
    Projectors --> Vector
    Queries --> Postgres
    Queries --> Graph
    Queries --> Tantivy
    Queries --> Vector
    AgentRuntime --> Queries
    AgentRuntime --> Commands
    AgentRuntime --> Ollama
    Projectors --> Embeddings
    Projectors --> OCR
    PluginHost --> Commands
    PluginHost --> Queries
```

## Responsibilities

| Container | Responsibility |
| --- | --- |
| Tauri shell | desktop packaging, OS integration, secure local bridge |
| SvelteKit UI | user workflows, command palette, graph/search/timeline UX |
| API server | application boundary, auth/session, commands and queries |
| Ingestion workers | provider sync, normalization, source preservation |
| Projection workers | build relational, graph, search and semantic views |
| Agent runtime | plan and execute AI workflows with tool permissions |
| Plugin host | load bounded extensions with explicit capabilities |
| PostgreSQL | primary relational and event persistence |
| Tantivy | full text search index |
| Vector index | semantic retrieval |
| Object store | documents, attachments, extracted artifacts |
| Ollama | local LLM execution |
