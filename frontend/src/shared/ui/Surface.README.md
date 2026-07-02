# Surface

Lowest-level visual surface primitive.

Use for neutral page, panel, or local UI wrappers that need consistent tone,
padding, radius, and border treatment.

Do not use for domain-specific cards or provider/business state.

Clipping is off by default so overlays can escape their trigger surface. Use
`clip` only for media, masks, or intentionally clipped visuals.

Accessibility: choose the `as` prop to match the semantic role of the content.
