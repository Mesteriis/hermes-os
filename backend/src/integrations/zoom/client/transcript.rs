use super::*;

pub(crate) struct ZoomParsedTranscriptFile {
    pub(crate) transcript_text: String,
    pub(crate) segments: Value,
    pub(crate) format: String,
    pub(crate) parsed_segment_count: usize,
}

#[derive(Clone, Debug)]
struct TimedTranscriptSegment {
    start_ms: i64,
    end_ms: i64,
    text: String,
}

pub fn empty_json_object() -> Value {
    json!({})
}

pub fn empty_json_array() -> Value {
    json!([])
}

pub(super) fn trimmed_optional(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(super) fn has_optional_ref(value: &Option<String>) -> bool {
    trimmed_optional(value).is_some()
}

pub(super) fn validate_optional_ref(
    field: &'static str,
    value: &Option<String>,
) -> Result<(), ZoomError> {
    if value
        .as_ref()
        .is_some_and(|candidate| candidate.trim().is_empty())
    {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

pub(super) fn validate_refresh_threshold(value: Option<i64>) -> Result<(), ZoomError> {
    if let Some(seconds) = value
        && !(ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS
            ..=ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS)
            .contains(&seconds)
    {
        return Err(ZoomError::InvalidRequest(
            "refresh_expiring_within_seconds must be between 60 and 86400".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn parse_zoom_transcript_file(
    file_text: &str,
    file_name: Option<&str>,
    content_type: Option<&str>,
) -> Result<ZoomParsedTranscriptFile, ZoomError> {
    let normalized = normalize_transcript_newlines(file_text);
    let trimmed = normalized.trim_start_matches('\u{feff}').trim();
    validate_non_empty("file_text transcript content", trimmed)?;

    let timed_segments = parse_timed_transcript_segments(trimmed)?;
    if !timed_segments.is_empty() {
        let transcript_text = timed_segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(ZoomParsedTranscriptFile {
            transcript_text,
            segments: json!(
                timed_segments
                    .iter()
                    .map(|segment| json!({
                        "start_ms": segment.start_ms,
                        "end_ms": segment.end_ms,
                        "text": segment.text,
                    }))
                    .collect::<Vec<_>>()
            ),
            format: infer_timed_transcript_format(trimmed, file_name, content_type).to_owned(),
            parsed_segment_count: timed_segments.len(),
        });
    }

    if trimmed.contains("-->") {
        return Err(ZoomError::InvalidRequest(
            "timed transcript file did not contain parseable cues".to_owned(),
        ));
    }

    let transcript_text = plain_transcript_text(trimmed);
    validate_non_empty("file_text transcript content", &transcript_text)?;
    Ok(ZoomParsedTranscriptFile {
        transcript_text,
        segments: json!([]),
        format: "plain_text".to_owned(),
        parsed_segment_count: 0,
    })
}

pub(super) fn normalize_transcript_newlines(file_text: &str) -> String {
    file_text.replace("\r\n", "\n").replace('\r', "\n")
}

fn parse_timed_transcript_segments(
    transcript_text: &str,
) -> Result<Vec<TimedTranscriptSegment>, ZoomError> {
    let mut blocks: Vec<Vec<&str>> = Vec::new();
    let mut current_block: Vec<&str> = Vec::new();
    for line in transcript_text.lines() {
        if line.trim().is_empty() {
            if !current_block.is_empty() {
                blocks.push(std::mem::take(&mut current_block));
            }
        } else {
            current_block.push(line);
        }
    }
    if !current_block.is_empty() {
        blocks.push(current_block);
    }

    let mut segments = Vec::new();
    for block in blocks {
        let Some(timing_index) = block.iter().position(|line| line.contains("-->")) else {
            continue;
        };
        let (start_ms, end_ms) = parse_timing_line(block[timing_index])?;
        let cue_text = block
            .iter()
            .skip(timing_index + 1)
            .map(|line| clean_cue_text_line(line))
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if cue_text.trim().is_empty() {
            continue;
        }
        segments.push(TimedTranscriptSegment {
            start_ms,
            end_ms,
            text: cue_text,
        });
    }
    Ok(segments)
}

pub(super) fn parse_timing_line(line: &str) -> Result<(i64, i64), ZoomError> {
    let (start, end_with_settings) = line.split_once("-->").ok_or_else(|| {
        ZoomError::InvalidRequest("transcript cue timing must contain -->".to_owned())
    })?;
    let end = end_with_settings.split_whitespace().next().ok_or_else(|| {
        ZoomError::InvalidRequest("transcript cue end time is required".to_owned())
    })?;
    let start_ms = parse_transcript_timestamp_ms(start.trim()).ok_or_else(|| {
        ZoomError::InvalidRequest(format!(
            "invalid transcript cue start time `{}`",
            start.trim()
        ))
    })?;
    let end_ms = parse_transcript_timestamp_ms(end.trim()).ok_or_else(|| {
        ZoomError::InvalidRequest(format!("invalid transcript cue end time `{}`", end.trim()))
    })?;
    if end_ms < start_ms {
        return Err(ZoomError::InvalidRequest(
            "transcript cue end time must be greater than or equal to start time".to_owned(),
        ));
    }
    Ok((start_ms, end_ms))
}

pub(super) fn parse_transcript_timestamp_ms(raw: &str) -> Option<i64> {
    let normalized = raw.trim().replace(',', ".");
    let parts = normalized.split(':').collect::<Vec<_>>();
    let (hours, minutes, seconds) = match parts.as_slice() {
        [minutes, seconds] => (0, minutes.parse::<i64>().ok()?, *seconds),
        [hours, minutes, seconds] => (
            hours.parse::<i64>().ok()?,
            minutes.parse::<i64>().ok()?,
            *seconds,
        ),
        _ => return None,
    };
    if hours < 0 || minutes < 0 {
        return None;
    }
    let (seconds, millis) = parse_seconds_and_millis(seconds)?;
    if seconds < 0 {
        return None;
    }
    Some((((hours * 60) + minutes) * 60 + seconds) * 1000 + millis)
}

pub(super) fn parse_seconds_and_millis(raw: &str) -> Option<(i64, i64)> {
    let (seconds, fraction) = raw.split_once('.').unwrap_or((raw, ""));
    let seconds = seconds.parse::<i64>().ok()?;
    if fraction.is_empty() {
        return Some((seconds, 0));
    }
    if !fraction.chars().all(|value| value.is_ascii_digit()) {
        return None;
    }
    let mut millis = fraction.chars().take(3).collect::<String>();
    while millis.len() < 3 {
        millis.push('0');
    }
    Some((seconds, millis.parse::<i64>().ok()?))
}

pub(super) fn clean_cue_text_line(line: &str) -> String {
    html_unescape_minimal(&strip_markup_tags(line.trim()))
        .trim()
        .to_owned()
}

pub(super) fn strip_markup_tags(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output
}

pub(super) fn html_unescape_minimal(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

pub(super) fn infer_timed_transcript_format(
    transcript_text: &str,
    file_name: Option<&str>,
    content_type: Option<&str>,
) -> &'static str {
    let file_name = file_name.unwrap_or_default().trim().to_ascii_lowercase();
    let content_type = content_type.unwrap_or_default().trim().to_ascii_lowercase();
    if transcript_text.trim_start().starts_with("WEBVTT")
        || file_name.ends_with(".vtt")
        || content_type.contains("webvtt")
    {
        return "webvtt";
    }
    if file_name.ends_with(".srt") || content_type.contains("srt") {
        return "srt";
    }
    "timed_text"
}

pub(super) fn plain_transcript_text(transcript_text: &str) -> String {
    transcript_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| *line != "WEBVTT")
        .collect::<Vec<_>>()
        .join("\n")
}
