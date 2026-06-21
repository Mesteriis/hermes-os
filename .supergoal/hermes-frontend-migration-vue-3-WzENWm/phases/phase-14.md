SUPERGOAL_PHASE_START
Phase: 14 of 15 — Polish & Harden
Task: Catch what earlier phases missed — UX copy, states, edges, security, a11y, perf, animations, and regression sweep
Mandatory commands: cd frontend && pnpm build, cd frontend && pnpm lint (if configured)
Acceptance criteria: 9
Evidence required: one paragraph per sub-pass, bundle size analysis, final screenshots
Depends on phases: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13

## Why

Earlier phases were focused on shipping behavior — getting each domain ported and rendering. This phase enforces that every aspect is production-quality: UX copy, empty/loading/error/unauthorized states, edge cases, security, accessibility, performance, animations, and a full regression sweep against the existing Hermes SvelteKit app.

## Work

### Sub-pass 1: UX & Copy

- Audit every visible string across all 14 ported domains
- Remove any debug placeholders, TODO stubs, or lorem-ipsum content
- Verify all strings use the i18n system (`useI18n().t()`) — no hardcoded Russian or English strings
- Check that error messages are user-friendly, not raw API errors or stack traces
- Verify button labels, tooltips, aria-labels, empty-state messages, and toast notifications
- Run: `grep -r "TODO\|FIXME\|HACK\|XXX\|debug\|placeholder\|lorem\|stub" frontend/src/ --include="*.vue" --include="*.ts"` — review each match

### Sub-pass 2: States

- For EVERY domain surface (14 domains), verify these states:
  - **Loading**: Skeleton or spinner shown during TanStack Query `isPending`
  - **Empty**: Meaningful empty state message with icon when data array is empty
  - **Error**: Error state with retry button when TanStack Query `isError`
  - **Unauthorized**: Proper handling when API returns 401/403 (redirect to auth or show message)
- Create or update a `frontend/src/domains/<domain>/components/States.vue` pattern if needed
- Verify optimistic updates in TanStack Query mutations handle rollback gracefully

### Sub-pass 3: Edges

- Test with:
  - Empty inputs (empty strings, zero-length arrays)
  - Very long inputs (1000+ character strings, long names)
  - Special characters (Unicode, emoji, HTML injection attempts, SQL-like patterns)
  - Slow network (simulate via browser DevTools throttling)
  - Rapid repeated clicks (debounce/throttle on save/submit buttons)
- Verify forms disable submit button while mutation is pending
- Verify file uploads handle cancellation, large files, and wrong file types

### Sub-pass 4: Security

- Verify `X-Hermes-Secret` header is present in ALL API calls (not just get/post — every method)
- Check that no secrets, tokens, or private data appear in:
  - Compiled bundle (`grep -r "password\|secret\|token\|api_key" frontend/dist/ --include="*.js"` after build)
  - Console.log statements in production
  - Error messages exposed to user
- Verify input validation/sanitization on all user-input forms
- Check that sandboxed iframe for message viewer (Communications) has proper `sandbox` attribute
- Verify CSP is not wide-open (check tauri.conf.json and any meta tags)

### Sub-pass 5: A11y

- Keyboard navigation:
  - Tab through all interactive elements — focus order must be logical
  - All dialogs trap focus while open
  - Escape closes dialogs, drawers, popovers
  - Enter/Space activates buttons and links
- Focus management:
  - Focus returns to trigger element after dialog/drawer closes
  - Route changes focus to main content area or h1
- Screen reader:
  - All images have `alt` text or `aria-hidden="true"`
  - All interactive elements have accessible names
  - ARIA live regions for dynamic content updates (toast notifications, SSE updates)
  - Proper heading hierarchy (h1 → h2 → h3, no skipping)
- Contrast: verify all text/background combinations meet WCAG AA (4.5:1 for normal text, 3:1 for large)

### Sub-pass 6: Performance

- **Virtual scrolling**: Verify TanStack Virtual is used for ALL large collections:
  - Communications mail list
  - Telegram chat list, WhatsApp session list
  - Documents list, Notes list
  - Tasks list
  - Timeline stream
  - Personas list
  - Projects list
- **Bundle size analysis**: Run `cd frontend && pnpm build && du -sh dist/` and `ls -lh dist/assets/`
- **No N+1 queries**: Verify TanStack Query patterns — no queries inside loops
- **Lazy loading**: Route-level code splitting via Vue Router dynamic imports
- **Image optimization**: Verify images are properly sized and lazy-loaded

### Sub-pass 7: Diff Review

- Search for and clean up:
  - Stray `console.log()` statements (use `console.debug()` or a logger utility instead)
  - `TODO` comments left from migration phases
  - Unused imports (run `eslint --rule 'unused-imports/no-unused-imports: error'` or similar)
  - Dead code/commented-out blocks
  - Duplicate type definitions
- Run `cd frontend && pnpm build` to confirm no warnings

### Sub-pass 8: Regression Sweep

- Full build: `cd frontend && pnpm build` — confirm exits 0, no warnings
- Visual comparison: Open both the new Vue app and the existing SvelteKit app (from `frontend/src-svelte/` if still runnable, or reference screenshots)
- Compare key surfaces side-by-side:
  - Home dashboard layout and widget rendering
  - Settings panel appearance
  - Mail list and message viewer
  - Persona detail view
  - Knowledge graph
- Verify no visual regressions: colors, spacing, typography, shadows, border radii, transitions

### Sub-pass 9: Animation

- Verify workspace transitions (route changes) animate smoothly via Motion library
- Panel animations (sidebar open/close, drawer slide-in) are smooth
- Micro-interactions:
  - Button hover/active states
  - Checkbox/switch toggle animations
  - List item hover effects
  - Dialog open/close transitions
- Verify animations respect `prefers-reduced-motion` — disable animations when user prefers reduced motion
- Ensure animations are not too slow (max 300ms for UI transitions)

## Acceptance criteria

- [ ] AC1: UX audit complete — no debug placeholders, all strings through i18n
- [ ] AC2: All 14 domains verified for loading/empty/error/unauthorized states
- [ ] AC3: Edge case testing completed — empty inputs, long inputs, special chars, slow network
- [ ] AC4: Security audit passed — X-Hermes-Secret in all calls, no secrets in bundle
- [ ] AC5: A11y audit passed — keyboard nav, focus management, headings, contrast ≥ AA
- [ ] AC6: Performance verified — virtual scrolling on all large collections, bundle size acceptable
- [ ] AC7: Diff review clean — no console.log, no TODOs, no unused imports
- [ ] AC8: Regression sweep passed — visual comparison confirms no regressions
- [ ] AC9: Animation pass — transitions smooth, respects prefers-reduced-motion

## Mandatory commands

- `cd frontend && pnpm build`
- `grep -r "TODO\|FIXME\|HACK\|XXX\|console\.log" frontend/src/ --include="*.vue" --include="*.ts"` (review output, clean up)
- `cd frontend && du -sh dist/` and `ls -lh dist/assets/` (bundle size check)

## Evidence required in transcript

- One paragraph per sub-pass — what was checked, what was found, what was fixed
- Bundle size analysis output
- Screenshots of 3 key surfaces: Home, Settings, Communications (or relevant domain)
- List of any remaining known issues or deferred work

## Notes

- This phase is intentionally manual-intensive. Each sub-pass requires human judgment, not just automated checks.
- If any sub-pass reveals systemic issues (e.g., all domains missing empty states), create a shared component fix rather than fixing each domain individually.
- The regression sweep should use the same test data/account for both old and new apps.
- Document any visual differences found and whether they are acceptable regressions or improvements.
- Proceed to Phase 15 (Cutover) only when all 9 acceptance criteria are satisfied.
