# Google OAuth Desktop Client Resource

This directory is packaged into the Tauri bundle as `$RESOURCES/google-oauth/`.

Release builds must place the Google OAuth Desktop app JSON at:

- `client_secret.json`

Use:

```sh
make google-oauth-resource
make frontend-tauri-build
```

`make google-oauth-resource` copies the JSON from
`HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH` in `docker/.env`, or from
`HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE` when set in the shell.

The generated `client_secret.json` is ignored by Git. It is intentionally a
bundle artifact, not a source-controlled credential file. Packaged builds pass
the bundled file path to the backend sidecar as
`HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH`.
