# AsyncSelect

UI-only async state wrapper for externally loaded select options.

It receives `loading`, `error` and `options` from a parent. It never performs a
network request and emits `search` / `retry` intent only.

Boundary rules:

- wraps `SearchableSelect`;
- does not fetch, cache, debounce or own async data;
- disables selection while `disabled`, `loading` or `error` is active;
- uses parent-provided labels for user-visible async state text.

## Props

- `modelValue?: string` - selected option value.
- `options?: SelectOption[]` - externally loaded options.
- `placeholder?: string` - trigger text when nothing is selected.
- `searchPlaceholder?: string` - search input placeholder.
- `ariaLabel?: string` - accessible trigger label.
- `disabled?: boolean` - disables interaction.
- `loading?: boolean` - shows loading state and disables the select.
- `loadingLabel?: string` - localized loading state label; defaults to
  `Loading options` for accessibility and should be overridden from i18n by
  product surfaces.
- `error?: string` - non-empty error text shown below the select.
- `emptyLabel?: string` - empty state shown after filtering.
- `retryLabel?: string` - retry button label.
- `class?: string` - extra root class.

## Events

- `update:modelValue` emits the selected option value.
- `search` emits the local search query from `SearchableSelect`.
- `select` emits the selected option.
- `clear` emits when the current selection is cleared.
- `retry` emits when the retry button is pressed.
