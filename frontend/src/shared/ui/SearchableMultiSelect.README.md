# SearchableMultiSelect

Multi-value select with local search, removable chips, select-all and clear-all
actions.

Boundary rules:

- accepts local `SelectOption[]`;
- emits selected values;
- does not fetch data;
- does not own domain validation.

## Props

- `modelValue?: string[]` - selected option values.
- `options?: SelectOption[]` - local options from `Selection.types`.
- `placeholder?: string` - trigger text when no known options are selected.
- `searchPlaceholder?: string` - search input placeholder.
- `ariaLabel?: string` - accessible label for the combobox trigger.
- `searchAriaLabel?: string` - accessible label for the search input.
- `listboxAriaLabel?: string` - accessible label for the option listbox.
- `actionsAriaLabel?: string` - accessible label for select-all and clear-all actions.
- `removeLabel?: (option: SelectOption) => string` - accessible label for chip removal.
- `selectedCountLabel?: (count: number) => string` - visible trigger label for multiple selections.
- `disabled?: boolean` - disables all interaction.
- `readonly?: boolean` - prevents mutation and opening.
- `emptyLabel?: string` - empty state shown after filtering.
- `selectAllLabel?: string` - bulk-select action label.
- `clearAllLabel?: string` - bulk-clear action label.
- `class?: string` - extra root class.

## Events

- `update:modelValue` emits the next selected value list.
- `search` emits the local search query.
- `select` emits an option when it is added to the selection.
- `clear` emits when all selections are cleared.
- `open` and `close` emit only on actual open-state transitions.

## Keyboard and Accessibility

- The trigger uses combobox semantics and controls a multiselect listbox.
- Arrow keys move the active option while keeping it visible.
- `Enter` and `Space` toggle the active option.
- `Escape` closes the popover.
- Focus leaving the component closes the popover.
