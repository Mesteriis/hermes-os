# Paper

Elevated document-like surface primitive.

Use for focused readable content, review summaries, and bounded local UI areas.

Do not use for every page section or as a generic layout wrapper.

Clipping is off by default so overlays can escape the paper surface. Use `clip`
only for media, masks, or intentional visual clipping.

Accessibility: choose semantic `as` values and avoid hiding focusable content inside decorative wrappers.
