---
batch_id: batch-20260628T214902
generated_by: code-wiki-ru
---

# Генерация wiki

```mermaid
flowchart LR
  index["repo-index.json"] --> packs["163 context packs"]
  packs --> opencode["OpenCode bounded agent"]
  opencode --> deepseek["DeepSeek drafts"]
  deepseek --> review["host apply/review"]
  review --> wiki["hierarchical Obsidian wiki"]
  wiki --> drift["_meta/drift-report.md"]
```

- Batch: `batch-20260628T214902`
- Context packs: `163`
- Drafts applied into detail pages: `163`
- Raw drafts remain under `_meta/drafts/batch-20260628T214902` for audit.
