# Yandex Telemost Implementation Plan

## Stage 1 - Foundation

- Add provider kind `yandex_telemost_user`.
- Add secret purpose `yandex_telemost_oauth_token`.
- Add integration module and error model.
- Add account setup with HostVault token storage.
- Add runtime status/capabilities route.

## Stage 2 - Conference API

- Add create conference.
- Add read conference.
- Add update conference.
- Add cohost list read.
- Emit sanitized integration events.

## Stage 3 - Desktop companion

- Add visible Telemost WebView.
- Add origin/navigation guard.
- Add WebView active-speaker hint bridge.
- Add local MP3 recorder using explicit ffmpeg process.
- Add speaker timeline JSONL/TXT output.
- Add consent gate before recording.

## Stage 4 - Provider-neutral projection

- Listen to `integration.yandex_telemost.conference.*` events.
- Project conference evidence into provider-neutral Calls/Calendar link model.
- Create Radar signals for unmatched meeting links, live streams, unknown cohosts
  and local recording artifacts.
- Mirror Radar signals into the Review Inbox as observation-backed candidates
  before any owner-domain promotion.
- Stamp matched `calendar_event_id` into the Call Bundle when the conference URL
  resolves to an existing calendar event, so later transcript/recording workflows
  do not emit false unmatched-link signals.

## Stage 5 - Transcription workflow

- Import `audio.mp3` as document/call evidence.
- Run transcription/diarization.
- Use `speaker-timeline.jsonl` and `speaker-timeline.txt` only as hints.
- Store transcript with Source, Confidence and Evidence.
- Accept transcript completion through a provider-neutral runtime bridge and
  project the completed transcript into `documents` as evidence-backed meeting memory.

## Explicit non-goals

- No hidden recording.
- No direct integration-to-domain mutation.
- No raw OAuth token in app responses.
- No belief that a DOM class called `active` is a person speaking with divine
  certainty. Computers lie. Web UIs lie more.
