# StatCard

Generic statistic surface for a single label, value, optional trend, and
optional description.

Use for compact dashboard or summary metrics where the calling feature already
computed the display value.

Do not add domain fields, fetch data, derive calculations, or encode product
semantics in this primitive.

Accessibility: keep `label` explicit and make the rendered value meaningful as
plain text.
