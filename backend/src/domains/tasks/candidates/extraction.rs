use serde_json::json;

use crate::engines::obligation::models::ObligationCandidate;

use super::constants::{
    OBLIGATION_CANDIDATE_METADATA_KEY, REVIEW_TEXT_SNIPPET_CHARS, TITLE_PREVIEW_CHARS,
};
use super::errors::TaskCandidateError;
use super::models::{CandidatePayload, TaskCandidateKind, TaskCandidateSourceKind};
use super::validation::text_preview;

const ACTION_MARKERS: &[&str] = &[
    "action:",
    "action required",
    "please ",
    "follow up",
    "next step",
    "действие:",
    "надо ",
    "нужно ",
    "следующий шаг",
    "accion:",
    "acción:",
    "por favor",
    "seguimiento",
    "siguiente paso",
    "proximo paso",
    "próximo paso",
    "tarea:",
    "hay que ",
    "tenemos que ",
    "necesito que ",
    "a faire:",
    "à faire:",
    "merci de ",
    "veuillez ",
    "suivi",
    "prochaine etape",
    "prochaine étape",
    "il faut ",
    "aktion:",
    "aufgabe:",
    "bitte ",
    "nachfassen",
    "naechster schritt",
    "nächster schritt",
    "wir muessen ",
    "wir müssen ",
];

const FREEFORM_REQUEST_MARKERS: &[&str] = &[
    "can you ",
    "could you ",
    "would you ",
    "можешь ",
    "можете ",
    "сможешь ",
    "puedes ",
    "podrias ",
    "podrías ",
    "peux-tu ",
    "pouvez-vous ",
    "pourrais-tu ",
    "kannst du ",
    "koenntest du ",
    "könntest du ",
];

const TASK_INTENT_TERMS: &[&str] = &[
    "check",
    "review",
    "prepare",
    "send",
    "schedule",
    "update",
    "confirm",
    "draft",
    "create",
    "fix",
    "investigate",
    "провер",
    "подготов",
    "отправ",
    "запланир",
    "обнов",
    "подтверд",
    "сдела",
    "preparar",
    "revisar",
    "comprobar",
    "enviar",
    "programar",
    "actualizar",
    "confirmar",
    "préparer",
    "preparer",
    "vérifier",
    "verifier",
    "envoyer",
    "planifier",
    "confirmer",
    "prévo",
    "prevo",
    "prüfen",
    "pruefen",
    "vorbereiten",
    "senden",
    "planen",
    "aktualisieren",
    "bestätigen",
    "bestaetigen",
];

const DUE_SEPARATORS: &[&str] = &[" до ", " by ", " para ", " avant ", " bis "];

pub(crate) struct CandidateFragment {
    pub(crate) text: String,
    pub(crate) due_text: Option<String>,
    pub(crate) assignee_label: Option<String>,
}

pub(crate) fn extract_candidate_fragment(text: &str) -> Option<CandidateFragment> {
    let text_lower = text.to_lowercase();
    if !contains_task_signal(&text_lower) {
        return None;
    }

    let selected = text
        .lines()
        .map(str::trim)
        .find(|line| {
            let lower = line.to_lowercase();
            contains_task_signal(&lower)
        })
        .unwrap_or(text);

    let due_text = text.lines().find_map(parse_due_text);
    let assignee_label = text.lines().find_map(parse_assignee_label);

    Some(CandidateFragment {
        text: selected.to_owned(),
        due_text,
        assignee_label,
    })
}

fn contains_task_signal(normalized: &str) -> bool {
    contains_action_marker(normalized) || contains_freeform_task_request(normalized)
}

fn contains_action_marker(normalized: &str) -> bool {
    ACTION_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn contains_freeform_task_request(normalized: &str) -> bool {
    FREEFORM_REQUEST_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        && TASK_INTENT_TERMS
            .iter()
            .any(|term| normalized.contains(term))
}

pub(crate) fn parse_due_text(line: &str) -> Option<String> {
    let normalized = line.trim().to_lowercase();
    if normalized.starts_with("due") || normalized.starts_with("deadline") {
        return normalized.split_once(':').and_then(|(_, right)| {
            let due = right.trim();
            (!due.is_empty()).then_some(due.to_owned())
        });
    }

    DUE_SEPARATORS.iter().find_map(|separator| {
        normalized.split_once(separator).and_then(|(_, right)| {
            let due = right
                .trim()
                .trim_end_matches(['.', '?', '!', ':', ';'])
                .trim();
            (!due.is_empty()).then_some(due.to_owned())
        })
    })
}

pub(crate) fn parse_assignee_label(line: &str) -> Option<String> {
    let normalized = line.to_lowercase();
    if !normalized.starts_with("assignee") {
        return None;
    }

    normalized.split_once(':').and_then(|(_, right)| {
        let assignee = right.trim();
        (!assignee.is_empty()).then_some(assignee.to_owned())
    })
}

pub(crate) fn title_from_fragment(value: &str) -> String {
    text_preview(value, TITLE_PREVIEW_CHARS)
}

pub(crate) fn evidence_excerpt(value: &str) -> String {
    text_preview(value, REVIEW_TEXT_SNIPPET_CHARS)
}

pub(crate) fn task_candidate_payload_from_obligation(
    observation_id: String,
    candidate: &ObligationCandidate,
) -> CandidatePayload {
    CandidatePayload {
        source_kind: TaskCandidateSourceKind::Observation,
        source_id: observation_id.clone(),
        observation_id: Some(observation_id),
        candidate_kind: TaskCandidateKind::ObligationTask,
        candidate_metadata: json!({
            "engine": "obligation",
            OBLIGATION_CANDIDATE_METADATA_KEY: candidate,
        }),
        project_id: None,
        title: title_from_fragment(&candidate.statement),
        due_text: candidate.due_text.clone(),
        assignee_label: None,
        confidence: (candidate.confidence - 0.08).max(0.0),
        evidence_excerpt: evidence_excerpt(&candidate.quote),
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_candidate_fragment, parse_due_text};

    #[test]
    fn extracts_multilingual_action_fragments() {
        let cases = [
            ("Acción: preparar el resumen para mañana.", Some("mañana")),
            ("À faire: préparer le résumé avant demain.", Some("demain")),
            (
                "Aufgabe: Zusammenfassung vorbereiten bis morgen.",
                Some("morgen"),
            ),
        ];

        for (text, expected_due_text) in cases {
            let fragment = extract_candidate_fragment(text)
                .unwrap_or_else(|| panic!("expected candidate fragment for {text}"));
            assert_eq!(fragment.text, text);
            assert_eq!(fragment.due_text.as_deref(), expected_due_text);
        }
    }

    #[test]
    fn extracts_freeform_multilingual_task_requests() {
        let cases = [
            (
                "Could you check the backup retention by Friday?",
                Some("friday"),
            ),
            (
                "Можешь проверить backup retention до пятницы?",
                Some("пятницы"),
            ),
            ("¿Puedes preparar el resumen para mañana?", Some("mañana")),
            ("Peux-tu préparer le résumé avant demain?", Some("demain")),
            (
                "Kannst du die Zulip-Zusammenfassung prüfen bis morgen?",
                Some("morgen"),
            ),
        ];

        for (text, expected_due_text) in cases {
            let fragment = extract_candidate_fragment(text)
                .unwrap_or_else(|| panic!("expected free-form candidate fragment for {text}"));
            assert_eq!(fragment.text, text);
            assert_eq!(fragment.due_text.as_deref(), expected_due_text);
        }
    }

    #[test]
    fn parses_common_due_separators() {
        let cases = [
            ("Action: prepare summary by Friday.", Some("friday")),
            ("Acción: preparar resumen para mañana.", Some("mañana")),
            ("À faire: préparer le résumé avant demain.", Some("demain")),
            (
                "Aufgabe: Zusammenfassung vorbereiten bis morgen.",
                Some("morgen"),
            ),
        ];

        for (line, expected_due_text) in cases {
            assert_eq!(parse_due_text(line).as_deref(), expected_due_text);
        }
    }
}
