SUPERGOAL_PHASE_START
Phase: 2 of 15 — App Shell
Task: Port sidebar, topbar, workspace layout, notifications drawer, layout editor to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 10
Evidence required: build output, route config, shell component listing
Depends on phases: 1

## Why

The app shell (sidebar, topbar, workspace layout) is the container for all domain views. Every domain renders inside it. Must be ported before any domain can render.

## Work

1. **Create Vue Router config with all route definitions:**
   - Create `frontend/src/app/router.ts`
   - Define routes for all 16 views: home, communications, persons, projects, tasks, calendar, documents, notes, knowledge, review, settings, agents, organizations, timeline, telegram, whatsapp
   - Each route maps to a placeholder component initially (actual domain components added in later phases)
   - Use Vue Router's history mode (hash mode for Tauri compatibility)

2. **Port Pinia stores from existing Svelte stores:**
   - Study existing Svelte stores in `frontend/src-svelte/lib/stores/`:
     - `navigation.ts` — activeView, activeCommunicationSection, isSidebarRail, expandedSidebarGroupIds, isUserMenuOpen, shellViewClass
     - `theme.ts` — shellThemeClass
     - `sidebar.ts` — sidebarRootEntries
     - `notifications.ts` — notificationItems, notificationCount, toggleNotificationsDrawer, openNotificationTarget
     - `layoutEditor.ts` — isLayoutEditing, activeWidgetById, visibleWidgetIds, layoutDraft, addableWidgetsForCurrentView, isWidgetDrawerOpen, selectedLayoutWidget, widgetGridValue, etc.
   - Create Pinia stores in `frontend/src/shared/stores/`:
     - `navigation.ts`, `theme.ts`, `sidebar.ts`, `notifications.ts`, `layoutEditor.ts`
   - Each store uses `defineStore` with Composition API
   - Server-derived state must NOT be in Pinia (only UI state)

3. **Port Sidebar component:**
   - Study `frontend/src-svelte/lib/components/shell/Sidebar.svelte`
   - Create `frontend/src/app/shell/Sidebar.vue`
   - Navigation groups with expand/collapse
   - Sidebar rail mode (compact/expanded toggle)
   - Active state highlighting
   - Settings button at bottom
   - Iconify icons for navigation items

4. **Port Topbar component:**
   - Study `frontend/src-svelte/lib/components/shell/Topbar.svelte`
   - Create `frontend/src/app/shell/Topbar.vue`
   - View title and subtitle display
   - Notification bell with count badge
   - User menu dropdown
   - Layout editing toggle button
   - Locale switch button
   - Exit button

5. **Port NotificationsDrawer component:**
   - Study existing notification drawer
   - Create `frontend/src/app/shell/NotificationsDrawer.vue`
   - Slide-in drawer with notification list
   - Click to navigate to notification target
   - Empty state

6. **Port LayoutEditor controls:**
   - Study `frontend/src-svelte/lib/components/shared/LayoutEditControls.svelte`
   - Create `frontend/src/app/shell/LayoutEditControls.vue`
   - Add widget, cancel, reset, save buttons
   - Widget settings drawer
   - Add widget drawer

7. **Create AppShell layout component:**
   - Create `frontend/src/app/shell/AppShell.vue`
   - Contains Sidebar (left), workspace area (center with Topbar at top), notifications drawer (overlay)
   - Handles sidebar rail mode and vault gate mode
   - Renders `<router-view>` in workspace area
   - Includes vault onboarding gate placeholder

8. **Update App.vue:**
   - Update `frontend/src/app/App.vue` to use AppShell wrapping `<router-view>`

9. **Port shell CSS:**
   - Study `frontend/src-svelte/lib/styles/shell.css`, `shellTheme.css`, `app.css`, `panels.css`
   - Create Tailwind-based shell styling that replicates the existing visual appearance
   - Use the Tailwind theme tokens from Phase 1

10. **Verify:**
    - `pnpm build` passes
    - Run `pnpm dev`, open browser, verify shell renders with sidebar, topbar, workspace
    - Test sidebar navigation between placeholder views
    - Test sidebar rail toggle
    - Test notification drawer open/close
    - Test layout editing mode toggle

## Acceptance criteria (all must pass)

- [ ] AC1: AppShell renders Sidebar, Topbar, and workspace router-view area
- [ ] AC2: Sidebar navigation switches between placeholder route views
- [ ] AC3: Topbar shows view title and notification count badge
- [ ] AC4: NotificationsDrawer opens/closes with slide animation
- [ ] AC5: User menu opens/closes
- [ ] AC6: Layout editing mode toggle works
- [ ] AC7: Sidebar rail mode (compact/expanded) toggle works
- [ ] AC8: All CSS/styling matches existing Hermes visual identity (compare with screenshots)
- [ ] AC9: `cd frontend && pnpm build` exits 0
- [ ] AC10: Vue Router config defines all 16 route paths

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output last 10 lines
- Vue Router config showing all defined routes
- List of created Pinia store files
- List of shell component files

## Notes

- Reference the existing SvelteKit code in `frontend/src-svelte/` for exact component behavior
- Do NOT port vault onboarding or VaultOnboarding component in this phase — it will be ported later or kept as a placeholder
- Use `@iconify/vue` for all icons (replacing `@iconify/svelte`)
- Pinia stores must NOT contain server-derived data — only transient UI state
- Theme store should read from localStorage or backend settings (same as existing)
- The layout editor widget settings and add-widget drawers can be simplified stubs in this phase
