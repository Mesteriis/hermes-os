SUPERGOAL_PHASE_START
Phase: 9 of 10 — Mail Module Parity
Task: Achieve full parity with Outlook, Apple Mail, Thunderbird — rich compose, rules, templates, signatures, multi-account, IMAP sync improvements
Mandatory commands: cargo build && cargo test --all, cd frontend && pnpm build
Acceptance criteria: 10
Evidence required: build output, Mail parity checklist, screenshots
Depends on phases: Phase 3

## Why

Mail модуль GAP-10 требует паритет с Outlook/Apple Mail/Thunderbird для ключевых сценариев. Rich composition, rules, templates, signatures и multi-account sync необходимы для полноценного использования Hermes как primary mail client.

## Work

### Backend

1. **Rich HTML email composition API:**
   - HTML body validation endpoint
   - Inline image upload/hosting
   - Template rendering endpoint

2. **Rules/filters engine enhancements:**
   - Rule evaluation improvements
   - Rule ordering/priority
   - Rule test/debug endpoint

3. **Full-text search Tantivy integration:**
   - Index mail messages in Tantivy
   - Search by sender, subject, body, date range
   - Faceted search with filters

4. **Attachments management:**
   - Preview metadata endpoint
   - Inline image detection
   - Attachment download optimization

5. **Signature management enhancements:**
   - Multiple signatures per account
   - Signature selection per compose mode reply/forward

6. **Multi-account sync improvements:**
   - Background sync stability
   - Conflict resolution
   - Sync status improvements

### Frontend

7. **Rich compose editor:**
   - TipTap WYSIWYG editor integration
   - Formatting: bold, italic, underline, lists, links
   - Inline image insertion
   - Attachment upload UI
   - Signature selector

8. **Thread/conversation view:**
   - Vertical thread visualization
   - Expand/collapse messages
   - Quick reply in thread

9. **Rules/filters UI:**
   - Rule list with enable/disable toggle
   - Rule creation wizard
   - Condition builder sender, subject, date
   - Action selector move, label, delete, forward

10. **Search UI:**
    - Search bar with advanced filters
    - Search results with snippets
    - Filter by folder, date range, sender

11. **Attachments panel:**
    - Attachment list with preview icons
    - Download all button
    - Inline image display in message viewer

12. **Signature editor:**
    - Create/edit/delete signatures
    - Default signature per account
    - Rich text formatting

## Acceptance criteria (all must pass)

- [ ] AC1: Rich compose editor TipTap работает — formatting, inline images, attachments
- [ ] AC2: Thread/conversation view показывает полную переписку в thread layout
- [ ] AC3: Rules/filters создаются через UI и применяются к входящим сообщениям
- [ ] AC4: Full-text search возвращает результаты с highlights
- [ ] AC5: Attachments preview/download работают для всех типов
- [ ] AC6: Signature editor позволяет создавать/редактировать/выбирать подпись
- [ ] AC7: Multi-account sync стабилен — нет race conditions или deadlocks
- [ ] AC8: `cargo build && cargo test --all` passes
- [ ] AC9: `cd frontend && pnpm build` passes
- [ ] AC10: `make backend-validate` passes

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cargo build && cargo test --all`
- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output — last 10 lines from both backend and frontend
- Mail parity checklist — each item marked completed
- Screenshots of key mail features: compose editor, thread view, rules UI, search

## Notes

- Разбить на must-have и nice-to-have если объём слишком большой
- Must-have: rich compose, thread view, search, attachments
- Nice-to-have: rules UI, signature editor, sync improvements
- TipTap уже есть в dependencies — используйте @tiptap/vue-3
- Reference `docs/domains/communications.md` for Communication domain architecture
- Mail module parity — крупнейшая фаза по объёму; рассмотреть разбивку на sub-phases
