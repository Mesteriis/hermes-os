# Recording Bundle Contract

A Call Bundle is the durable evidence package for a live conversation.

## Layout

```text
call-bundle-{id}/
├── manifest.json
├── meeting.json
├── provider.json
├── participants.json
├── audio.mp3
├── speaker-hints.jsonl
├── speaker-timeline.txt
├── event-track.jsonl
├── chat.json
├── transcript.json
├── transcript.md
├── summary.md
├── topics.json
├── entities.json
├── decisions.json
├── tasks.json
├── knowledge.json
├── metrics.json
├── radar-signals.json
├── screenshots/
├── attachments/
└── ocr/
```

Only a subset exists immediately after recording. Later pipeline stages append
new artifacts.

## Manifest

```json
{
  "schema_version": 1,
  "bundle_id": "call_...",
  "provider_kind": "yandex_telemost",
  "provider_shape": "visible_webview_local_capture",
  "account_id": "...",
  "provider_conference_id": "...",
  "join_url": "...",
  "calendar_event_id": null,
  "project_id": null,
  "organization_id": null,
  "artifacts": [
    {
      "kind": "audio",
      "relative_path": "audio.mp3",
      "source": "local_audio_loopback",
      "truth_status": "capture_artifact",
      "media_type": "audio/mpeg"
    },
    {
      "kind": "speaker_hints",
      "relative_path": "speaker-hints.jsonl",
      "source": "visible_webview_dom_heuristic",
      "truth_status": "hint_not_truth",
      "media_type": "application/x-ndjson"
    }
  ]
}
```

## Artifact policy

| Artifact | Truth status | Purpose |
|---|---|---|
| `audio.mp3` | capture artifact | transcription, diarization |
| `speaker-hints.jsonl` | hint, not truth | warm start speaker assignment |
| `event-track.jsonl` | observed runtime events | meeting timeline |
| `chat.json` | provider/UI capture | communication context |
| `screenshots/*` | local visual evidence | OCR, screen intelligence |
| `transcript.json` | model output | searchable text with evidence links |
| `decisions.json` | candidate output | review/Radar/ADR candidates |
| `tasks.json` | candidate output | review/Radar/task candidates |

## Privacy and consent

A bundle may only be created from a visible user-owned session with explicit
recording consent. Hidden capture, silent device installation and background
meeting joins are forbidden.

The bundle should record privacy metadata:

```json
{
  "capture_mode": "visible_webview_local_loopback",
  "consent_attested": true,
  "hidden_capture": false,
  "provider_recording": false,
  "local_only": true
}
```

## Immutability

Raw artifacts should be append-only. Derived artifacts may be superseded by a
new version, but they should not overwrite the evidence they came from.
