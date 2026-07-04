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

Signal state: use `signal` when an external event should draw attention to an
existing card without changing its layout. Pair it with `signalTone` for
severity and `signalPulse=false` when motion should be disabled by the caller.
The component only renders the visual state; event timing and acknowledgement
belong to the owning surface.
