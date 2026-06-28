# Call Intelligence Engine

The Call Intelligence engine transforms a Call Bundle into structured,
evidence-backed candidates.

It does not create domain objects directly. It creates candidates that Radar,
Review or a workflow can promote later.

## Pipeline

```text
Call Bundle
↓
Artifact validation
↓
Transcription
↓
Diarization
↓
Speaker identity merge
↓
Topic timeline
↓
Entity extraction
↓
Decision detection
↓
Action item detection
↓
Knowledge extraction
↓
Radar projection
```

## Output classes

```text
summary
transcript segment
topic segment
speaker identity candidate
person candidate
organization candidate
project candidate
document reference
decision candidate
action item candidate
open question candidate
risk signal
radar signal
knowledge note candidate
```

## Candidate schema

```json
{
  "candidate_kind": "decision",
  "title": "Use provider-neutral Call Bundle for meeting memory",
  "confidence": 0.84,
  "evidence": {
    "bundle_id": "call_...",
    "transcript_segments": [42, 43],
    "time_range": {
      "start_ms": 912000,
      "end_ms": 972000
    },
    "source_artifacts": ["audio.mp3", "transcript.json"]
  }
}
```

## Decision detection

Decision candidates should capture durable commitments:

```text
We decided to use X.
We will not support Y.
The target architecture is Z.
This issue is accepted/rejected/deferred.
```

A decision candidate can later become:

```text
ADR candidate
project note
knowledge graph edge
meeting outcome
```

## Action detection

Action item candidates should preserve assignment uncertainty:

```json
{
  "candidate_kind": "action_item",
  "title": "Implement Telemost provider runtime",
  "assignee_hint": "Alex",
  "due_date_hint": null,
  "confidence": 0.77,
  "truth_status": "candidate"
}
```

The engine must not create a Task directly. It sends the candidate through
Radar/Review so the owning workflow can preserve source evidence and user
review semantics.

## Topic timeline

Topic segments allow replay and search by discussion area:

```json
{
  "title": "Audio capture architecture",
  "start_ms": 300000,
  "end_ms": 620000,
  "confidence": 0.81,
  "evidence_segments": [12, 13, 14, 15]
}
```

## Screen intelligence

When screenshots exist, the engine can trigger:

```text
OCR
UI detection
diagram detection
code snippet detection
document reference extraction
```

These outputs become knowledge candidates with screenshot/time evidence.
