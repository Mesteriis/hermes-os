# Frontend legacy reference

This tree is historical evidence only. It is not part of the Vite bundle,
active TypeScript graph, or frontend validation suite.

The archived Communications API specifications exercised handwritten
`ApiClient` and pre-clean-room Connect transport. Active browser code uses only
the same-origin Gateway boundary and does not retain that compatibility path.
Reintroducing a product route requires a new owner-specific generated Gateway
contract; these files must not be copied into active code as a fallback.

`app-shell-legacy/` and `communications-composition/` preserve the former
navbar aggregation and provider runtime composition as historical UI evidence.
They are deliberately excluded from the active browser entrypoint: the active
shell reads Kernel bootstrap availability and mounts only System Control until
an owner-specific generated Gateway contract is admitted.

The individual Mail, Telegram, WhatsApp, Calls, Meetings, Timeline, Zulip,
Slack, Discord and Mattermost surface factories are stored there with the
former aggregate. Compiled Storybook fixtures retain their visible UI inventory
without restoring provider runtime composition to the active client.

The former Communications page, Mail workspace adapter, message actions and
thread/compose orchestration are stored alongside them. The active client keeps
only clean presentation models and Kernel-admitted route composition.

The former WhatsApp panel and its provider-specific presentation adapter are
also historical reference; the active bundle does not open provider transport
or privileged host access for a route that Kernel has not admitted.
