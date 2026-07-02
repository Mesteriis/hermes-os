# ActionCard

Interactive card-shaped primitive for choosing or launching a local action.

Use for compact action choices where a button-like surface is clearer than a
standard text button.

Do not use as a passive content card or as a domain-specific workflow object.

Accessibility: the default render target is a native `button` with
`type="button"`. For non-button targets, disabled state is expressed with
`aria-disabled` and click handling suppresses default behavior and propagation.
