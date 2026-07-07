# Steps

Modal carousel wizard surface for short setup flows with explicit step
ownership.

Use `step-count` to declare how many steps exist and provide per-step content
through named slots: `#step-1`, `#step-2`, `#step-3`, and so on. The optional
`steps` prop supplies the active step subheader, supporting text, and dot
labels.

The component is controlled with `v-model:open` and `v-model:step`; owner
components remain responsible for validation and side effects.

The default footer has navigation only. A cancel action can be enabled with
`show-cancel` or fully replaced through the `footer` slot.

Accessibility: built on `Dialog`, exposes a labelled step navigation rail and a
region for the active step.

Example:

```vue
<Steps v-model:open="open" v-model:step="step" :step-count="3" :steps="steps">
	<template #step-1>Connection form</template>
	<template #step-2>Verification state</template>
	<template #step-3>Model selection</template>
</Steps>
```

Do not use for long domain workflows. Promote those to a page or workbench.
