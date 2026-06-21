SUPERGOAL_PHASE_START
Phase: 3 of 15 — Shared UI Primitives
Task: Initialize shadcn-vue components and build Level 1 shared UI primitives
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 8
Evidence required: build output, file listing of shared/ui/
Depends on phases: 1

## Why

UI primitives (Button, Input, Dialog, Dropdown, etc.) are used by every domain. Porting them early ensures consistency, avoids duplication, and establishes the Hermes-styled component library.

## Work

1. **Initialize shadcn-vue:**
   - Run `npx shadcn-vue init` to set up the components system
   - Configure to use the Hermes Tailwind theme
   - Components go to `frontend/src/shared/ui/`

2. **Add shadcn-vue components:**
   - Add each component via `npx shadcn-vue add <component>` or by manually creating them
   - Required components: Button, Input, Dialog, DropdownMenu, Select, Switch, Tabs, Card, Badge, Avatar, Tooltip, Popover, Command (for palette), Sheet (for drawers), Separator, ScrollArea, Skeleton, Progress, Toast, Label, Textarea, Form

3. **Customize components for Hermes visual identity:**
   - Modify each component's styling to use Hermes theme tokens (colors, spacing, typography, border radius)
   - Ensure components match the existing Hermes visual style, NOT default shadcn look
   - Key customizations:
     - Button: match existing Hermes button states (default, hover, active, disabled)
     - Input: match existing input styling (background, border, focus ring)
     - Dialog: match existing modal/drawer styling
     - Tabs: match existing tab styling

4. **Create barrel export:**
   - Create `frontend/src/shared/ui/index.ts` that exports all components

5. **Add Iconify Vue integration:**
   - Ensure `@iconify/vue` is installed and an `<IconifyIcon>` component is available
   - Create `frontend/src/shared/ui/Icon.vue` — wrapper for consistent icon usage

6. **Verify:**
   - Build passes
   - Each component renders correctly in a test view

## Acceptance criteria (all must pass)

- [ ] AC1: All required shadcn-vue components exist in `frontend/src/shared/ui/`
- [ ] AC2: Components use Hermes theme tokens (colors, spacing, typography)
- [ ] AC3: Button supports all variants (default, secondary, outline, ghost, destructive)
- [ ] AC4: Dialog opens/closes with smooth animation
- [ ] AC5: DropdownMenu opens on click with correct positioning
- [ ] AC6: Tooltip shows on hover with correct positioning
- [ ] AC7: `cd frontend && pnpm build` exits 0
- [ ] AC8: Icon wrapper works and renders Iconify icons correctly

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output last 10 lines
- List of files in `frontend/src/shared/ui/`
- Brief note on which components were customized from defaults

## Notes

- shadcn-vue components become project-owned code — modify them freely
- Do NOT depend on shadcn-vue's default styling; always customize to Hermes theme
- Icon wrapper should accept icon name string (e.g., "tabler:mail") and render the correct Iconify icon
- If `npx shadcn-vue add` has issues, create the component files manually based on shadcn-vue source
