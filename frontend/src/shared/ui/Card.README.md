# Card

Compact object or repeated-item surface primitive.

Use for item grids, summaries, and selectable or interactive local UI objects.

Do not use as a generic page section, layout wrapper, or domain-specific
business component.

Clipping is off by default so menus, popovers, and other overlays are not
constrained by the card. Use `clip` only for media, masks, or intentional visual
clipping.

Accessibility: choose semantic `as` values and use `variant="interactive"` only
when the card is meant to behave as an interactive surface.
