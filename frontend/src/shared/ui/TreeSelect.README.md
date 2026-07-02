# TreeSelect

Single-value select for local hierarchical option trees.

Use it when parent-child structure matters and a compact dropdown tree is enough.
Use `Cascader` when a deep hierarchy is easier to scan by columns.

## Behavior

- `modelValue` stores one leaf option `value`; option values must be globally unique across the tree.
- Parent values are not valid `modelValue` targets. Parents are expansion-only, so clicking a parent or pressing `Enter` / `Space` toggles expansion instead of selecting it.
- Leaf options are selectable and emit `update:modelValue` and `select`.
- Disabled options stay visible but cannot be selected or expanded. Descendants under a disabled parent are not auto-revealed for selection.
- The trigger emits `open` and `close` only when the open state changes.
- Focus leaving the component closes the popover.

## Keyboard

- `Escape` closes the tree.
- `ArrowUp` / `ArrowDown` move the active visible row and keep it scrolled into view.
- `ArrowRight` expands the active parent or moves into its first visible child when already expanded.
- `ArrowLeft` collapses the active parent or moves to the visible parent row.
- `Enter` / `Space` select the active leaf or toggle the active parent.
