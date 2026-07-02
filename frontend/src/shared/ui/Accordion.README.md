# Accordion

Controlled disclosure surface for compact groups of related content.

Use when several short sections compete for the same local space and the parent
owns open state through `modelValue`.

Do not use for navigation, tabs, or hidden data loading. The component has no
internal open state and only emits `update:modelValue`.

Accessibility: triggers are semantic buttons with `aria-expanded` and each open
panel is associated with its trigger.
