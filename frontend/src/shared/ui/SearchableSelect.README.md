# SearchableSelect

Single-value select with local search, clear action and keyboard navigation.

Props:

- `modelValue`: selected option value.
- `options`: local `SelectOption[]` source.
- `placeholder`, `searchPlaceholder`, `emptyLabel`: display text.
- `ariaLabel`: accessible trigger label.
- `clearLabel`: accessible label for the clear button.
- `searchAriaLabel`: accessible label for the search input.
- `disabled`, `readonly`, `clearable`: interaction controls.

Events:

- `update:modelValue`: selected value changes.
- `search`: local search query changes, including reset on close.
- `select`: enabled option is selected.
- `clear`: current selection is cleared.
- `open`, `close`: popover state transitions only.

Keyboard:

- `Enter`, `Space` or `ArrowDown` opens the list from the trigger.
- `ArrowUp` and `ArrowDown` move the active option.
- `Home` and `End` move the active option unless focus is in the search input.
- `Enter` selects the active option.
- `Escape` closes the list.

Accessibility:

- trigger uses combobox semantics with `aria-controls`, `aria-expanded` and `aria-activedescendant`;
- options keep selected state separate from active row state;
- search input has its own accessible label and shares the active descendant;
- empty state is presentational inside the listbox.

Boundary rules:

- accepts local `SelectOption[]`;
- emits search and selection intent;
- does not fetch data;
- does not import domains, integrations, stores or query clients.
