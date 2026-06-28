use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const PROVIDER_SHAPE: &str = "yandex_telemost_user";
const RUNTIME_KIND: &str = "yandex_telemost_webview_runtime";
const WINDOW_LABEL_PREFIX: &str = "yandex-telemost";
const TELEMOST_ALLOWED_HOST_RU: &str = "telemost.yandex.ru";
const TELEMOST_ALLOWED_HOST_COM: &str = "telemost.yandex.com";
const DEFAULT_FFMPEG_PATH: &str = "ffmpeg";
const DEFAULT_LINUX_MONITOR: &str = "hermes_telemost.monitor";

#[derive(Default)]
pub(crate) struct TelemostLocalRecorder {
    sessions: Mutex<HashMap<String, RecordingProcess>>,
}

impl Drop for TelemostLocalRecorder {
    fn drop(&mut self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            for (_, mut session) in sessions.drain() {
                let _ = session.child.kill();
                let _ = session.child.wait();
            }
        }
    }
}

struct RecordingProcess {
    child: Child,
    manifest: YandexTelemostRecordingSession,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostCompanionOpenRequest {
    pub(crate) account_id: String,
    pub(crate) join_url: String,
    #[serde(default)]
    pub(crate) conference_id: Option<String>,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostCompanionManifest {
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) join_url: String,
    pub(crate) provider_shape: &'static str,
    pub(crate) runtime_kind: &'static str,
    pub(crate) window_label: String,
    pub(crate) opened_window: bool,
    pub(crate) focused_existing_window: bool,
    pub(crate) owner_visible: bool,
    pub(crate) hidden_headless_mode: &'static str,
    pub(crate) allowed_hosts: Vec<&'static str>,
    pub(crate) speaker_timeline: SpeakerTimelineContract,
    pub(crate) recorder: LocalRecorderContract,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SpeakerTimelineContract {
    pub(crate) state: &'static str,
    pub(crate) source: &'static str,
    pub(crate) truth_status: &'static str,
    pub(crate) output_files: Vec<&'static str>,
    pub(crate) cadence_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalRecorderContract {
    pub(crate) state: &'static str,
    pub(crate) audio_format: &'static str,
    pub(crate) consent_attestation_required: bool,
    pub(crate) ffmpeg_path_env: &'static str,
    pub(crate) ffmpeg_input_env: &'static str,
    pub(crate) default_linux_input: &'static str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostAudioDevicePrepareRequest {
    #[serde(default)]
    pub(crate) device_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostAudioDeviceManifest {
    pub(crate) platform: &'static str,
    pub(crate) state: &'static str,
    pub(crate) input_hint: String,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostRecordingStartRequest {
    pub(crate) account_id: String,
    pub(crate) join_url: String,
    #[serde(default)]
    pub(crate) conference_id: Option<String>,
    #[serde(default)]
    pub(crate) window_label: Option<String>,
    #[serde(default)]
    pub(crate) audio_input: Option<String>,
    #[serde(default)]
    pub(crate) consent_attested: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostRecordingStopRequest {
    pub(crate) recording_session_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostRecordingSession {
    pub(crate) recording_session_id: String,
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) join_url: String,
    pub(crate) window_label: String,
    pub(crate) output_dir: String,
    pub(crate) audio_path: String,
    pub(crate) speaker_jsonl_path: String,
    pub(crate) speaker_txt_path: String,
    pub(crate) ffmpeg_pid: Option<u32>,
    pub(crate) started_at_epoch_ms: u128,
    pub(crate) consent_attested: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostRecordingStopReceipt {
    pub(crate) recording_session_id: String,
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) audio_path: String,
    pub(crate) speaker_jsonl_path: String,
    pub(crate) speaker_txt_path: String,
    pub(crate) stopped_at_epoch_ms: u128,
    pub(crate) state: &'static str,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct YandexTelemostSpeakerTimelineAppendRequest {
    pub(crate) account_id: String,
    #[serde(default)]
    pub(crate) conference_id: Option<String>,
    #[serde(default)]
    pub(crate) recording_session_id: Option<String>,
    pub(crate) speaker_label: String,
    #[serde(default)]
    pub(crate) confidence: Option<f32>,
    #[serde(default)]
    pub(crate) observed_at_epoch_ms: Option<u128>,
    #[serde(default)]
    pub(crate) source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostSpeakerTimelineAppendReceipt {
    pub(crate) recording_session_id: Option<String>,
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
}

#[tauri::command]
pub(crate) async fn yandex_telemost_companion_manifest(
    request: YandexTelemostCompanionOpenRequest,
) -> Result<YandexTelemostCompanionManifest, String> {
    validate_join_url(&request.join_url)?;
    let label = companion_window_label(&request.account_id, request.conference_id.as_deref())?;
    Ok(manifest_for_request(request, label, false, false))
}

#[tauri::command]
pub(crate) async fn open_yandex_telemost_companion(
    app: AppHandle,
    request: YandexTelemostCompanionOpenRequest,
) -> Result<YandexTelemostCompanionManifest, String> {
    validate_join_url(&request.join_url)?;
    let window_label =
        companion_window_label(&request.account_id, request.conference_id.as_deref())?;
    if let Some(window) = app.get_webview_window(&window_label) {
        window
            .show()
            .map_err(|error| format!("failed to show Telemost window: {error}"))?;
        window
            .set_focus()
            .map_err(|error| format!("failed to focus Telemost window: {error}"))?;
        return Ok(manifest_for_request(request, window_label, false, true));
    }

    let url = request
        .join_url
        .parse()
        .map_err(|error| format!("invalid Yandex Telemost join URL: {error}"))?;
    let initialization_script = telemost_initialization_script(&request, &window_label)?;
    let window = WebviewWindowBuilder::new(&app, window_label.clone(), WebviewUrl::External(url))
        .title("Yandex Telemost · Hermes")
        .visible(true)
        .resizable(true)
        .inner_size(1220.0, 820.0)
        .initialization_script(initialization_script)
        .on_navigation(|url| {
            url.scheme() == "https"
                && matches!(
                    url.host_str(),
                    Some(TELEMOST_ALLOWED_HOST_RU) | Some(TELEMOST_ALLOWED_HOST_COM)
                )
        })
        .build()
        .map_err(|error| format!("failed to open Yandex Telemost window: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("failed to focus Telemost window: {error}"))?;

    Ok(manifest_for_request(request, window_label, true, false))
}

#[tauri::command]
pub(crate) async fn yandex_telemost_prepare_audio_device(
    request: YandexTelemostAudioDevicePrepareRequest,
) -> Result<YandexTelemostAudioDeviceManifest, String> {
    prepare_audio_device(request)
}

#[tauri::command]
pub(crate) async fn yandex_telemost_recording_start(
    app: AppHandle,
    state: State<'_, TelemostLocalRecorder>,
    request: YandexTelemostRecordingStartRequest,
) -> Result<YandexTelemostRecordingSession, String> {
    if !request.consent_attested {
        return Err("recording requires explicit consent_attested=true; Hermes will not start hidden conference capture".to_owned());
    }
    validate_join_url(&request.join_url)?;
    let account_id = required_slug("account_id", &request.account_id)?;
    let window_label = match request
        .window_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => value.to_owned(),
        None => companion_window_label(account_id, request.conference_id.as_deref())?,
    };
    let session_id = recording_session_id(account_id, request.conference_id.as_deref());
    let output_dir = recording_output_dir(&app, account_id, &session_id)?;
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create Telemost recording dir: {error}"))?;
    let audio_path = output_dir.join("audio.mp3");
    let speaker_jsonl_path = output_dir.join("speaker-timeline.jsonl");
    let speaker_txt_path = output_dir.join("speaker-timeline.txt");
    write_timeline_header(&speaker_txt_path, &request, &session_id)?;
    let mut command = ffmpeg_recording_command(request.audio_input.as_deref(), &audio_path)?;
    let child = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start ffmpeg Telemost recorder: {error}"))?;
    let manifest = YandexTelemostRecordingSession {
        recording_session_id: session_id.clone(),
        account_id: account_id.to_owned(),
        conference_id: request.conference_id.clone(),
        join_url: request.join_url.clone(),
        window_label,
        output_dir: output_dir.to_string_lossy().into_owned(),
        audio_path: audio_path.to_string_lossy().into_owned(),
        speaker_jsonl_path: speaker_jsonl_path.to_string_lossy().into_owned(),
        speaker_txt_path: speaker_txt_path.to_string_lossy().into_owned(),
        ffmpeg_pid: Some(child.id()),
        started_at_epoch_ms: now_epoch_ms(),
        consent_attested: true,
    };
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "Telemost recorder state lock poisoned".to_owned())?;
    if sessions.contains_key(&session_id) {
        return Err(format!(
            "Telemost recording session `{session_id}` is already running"
        ));
    }
    sessions.insert(
        session_id,
        RecordingProcess {
            child,
            manifest: manifest.clone(),
        },
    );
    Ok(manifest)
}

#[tauri::command]
pub(crate) async fn yandex_telemost_recording_stop(
    state: State<'_, TelemostLocalRecorder>,
    request: YandexTelemostRecordingStopRequest,
) -> Result<YandexTelemostRecordingStopReceipt, String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "Telemost recorder state lock poisoned".to_owned())?;
    let mut process = sessions
        .remove(request.recording_session_id.trim())
        .ok_or_else(|| {
            format!(
                "Telemost recording session `{}` was not found",
                request.recording_session_id.trim()
            )
        })?;
    let _ = process.child.kill();
    let _ = process.child.wait();
    append_text_line(
        Path::new(&process.manifest.speaker_txt_path),
        &format!(
            "{}\tSYSTEM\trecording_stop\tconfidence=1.00\tsource=local_recorder",
            now_epoch_ms()
        ),
    )?;
    Ok(YandexTelemostRecordingStopReceipt {
        recording_session_id: process.manifest.recording_session_id,
        account_id: process.manifest.account_id,
        conference_id: process.manifest.conference_id,
        audio_path: process.manifest.audio_path,
        speaker_jsonl_path: process.manifest.speaker_jsonl_path,
        speaker_txt_path: process.manifest.speaker_txt_path,
        stopped_at_epoch_ms: now_epoch_ms(),
        state: "stopped",
    })
}

#[tauri::command]
pub(crate) async fn yandex_telemost_speaker_timeline_append(
    webview_window: WebviewWindow,
    state: State<'_, TelemostLocalRecorder>,
    request: YandexTelemostSpeakerTimelineAppendRequest,
) -> Result<YandexTelemostSpeakerTimelineAppendReceipt, String> {
    if !webview_window.label().starts_with(WINDOW_LABEL_PREFIX) {
        return Ok(YandexTelemostSpeakerTimelineAppendReceipt {
            recording_session_id: None,
            accepted: false,
            reason: "rejected_non_telemost_window",
        });
    }
    let sessions = state
        .sessions
        .lock()
        .map_err(|_| "Telemost recorder state lock poisoned".to_owned())?;
    let session = match request
        .recording_session_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(session_id) => sessions.get(session_id),
        None => sessions.values().find(|item| {
            item.manifest.account_id == request.account_id
                && item.manifest.conference_id == request.conference_id
        }),
    };
    let Some(session) = session else {
        return Ok(YandexTelemostSpeakerTimelineAppendReceipt {
            recording_session_id: request.recording_session_id,
            accepted: false,
            reason: "no_active_recording_session",
        });
    };
    let observed_at = request.observed_at_epoch_ms.unwrap_or_else(now_epoch_ms);
    let confidence = request.confidence.unwrap_or(0.35).clamp(0.0, 1.0);
    let speaker_label = sanitize_speaker_label(&request.speaker_label);
    let source = request.source.as_deref().unwrap_or("webview_dom_heuristic");
    let json_line = json!({
        "observed_at_epoch_ms": observed_at,
        "speaker_label": speaker_label,
        "confidence": confidence,
        "source": source,
        "truth_status": "hint_not_truth",
    });
    append_text_line(
        Path::new(&session.manifest.speaker_jsonl_path),
        &json_line.to_string(),
    )?;
    append_text_line(
        Path::new(&session.manifest.speaker_txt_path),
        &format!(
            "{}\t{}\tspeaker_hint\tconfidence={:.2}\tsource={}",
            observed_at, speaker_label, confidence, source
        ),
    )?;
    Ok(YandexTelemostSpeakerTimelineAppendReceipt {
        recording_session_id: Some(session.manifest.recording_session_id.clone()),
        accepted: true,
        reason: "speaker_hint_appended",
    })
}

fn manifest_for_request(
    request: YandexTelemostCompanionOpenRequest,
    window_label: String,
    opened_window: bool,
    focused_existing_window: bool,
) -> YandexTelemostCompanionManifest {
    YandexTelemostCompanionManifest {
        account_id: request.account_id,
        conference_id: request.conference_id,
        join_url: request.join_url,
        provider_shape: PROVIDER_SHAPE,
        runtime_kind: RUNTIME_KIND,
        window_label,
        opened_window,
        focused_existing_window,
        owner_visible: true,
        hidden_headless_mode: "forbidden",
        allowed_hosts: vec![TELEMOST_ALLOWED_HOST_RU, TELEMOST_ALLOWED_HOST_COM],
        speaker_timeline: SpeakerTimelineContract {
            state: "enabled_when_recording_session_is_active",
            source: "webview_dom_active_speaker_heuristic",
            truth_status: "hint_not_truth",
            output_files: vec!["speaker-timeline.jsonl", "speaker-timeline.txt"],
            cadence_ms: 1000,
        },
        recorder: LocalRecorderContract {
            state: "explicit_start_stop_only",
            audio_format: "mp3",
            consent_attestation_required: true,
            ffmpeg_path_env: "HERMES_TELEMOST_FFMPEG_PATH",
            ffmpeg_input_env: "HERMES_TELEMOST_FFMPEG_INPUT",
            default_linux_input: DEFAULT_LINUX_MONITOR,
        },
    }
}

fn telemost_initialization_script(
    request: &YandexTelemostCompanionOpenRequest,
    window_label: &str,
) -> Result<String, String> {
    let account_id =
        serde_json::to_string(&request.account_id).map_err(|error| error.to_string())?;
    let conference_id =
        serde_json::to_string(&request.conference_id).map_err(|error| error.to_string())?;
    let window_label = serde_json::to_string(window_label).map_err(|error| error.to_string())?;
    Ok(format!(
        r#"
(() => {{
  const ACCOUNT_ID = {account_id};
  const CONFERENCE_ID = {conference_id};
  const WINDOW_LABEL = {window_label};
  const ALLOWED = new Set(['{TELEMOST_ALLOWED_HOST_RU}', '{TELEMOST_ALLOWED_HOST_COM}']);
  if (!ALLOWED.has(window.location.hostname)) return;
  window.__HERMES_YANDEX_TELEMOST_COMPANION__ = {{
    providerShape: '{PROVIDER_SHAPE}',
    runtimeKind: '{RUNTIME_KIND}',
    windowLabel: WINDOW_LABEL,
    speakerTimelineTruthStatus: 'hint_not_truth'
  }};
  const normalize = (value) => String(value || '').replace(/\s+/g, ' ').trim().slice(0, 120);
  const candidateText = (el) => normalize(
    el.getAttribute('aria-label')
      || el.getAttribute('title')
      || el.getAttribute('data-participant-name')
      || el.getAttribute('data-display-name')
      || el.innerText
      || el.textContent
      || ''
  );
  const activeNeedle = /(speaking|speaker|active|говорит|говорящий|выступает)/i;
  const mutedNeedle = /(muted|микрофон выключен|без звука)/i;
  const selectors = [
    '[aria-current="true"]',
    '[aria-pressed="true"]',
    '[data-speaking="true"]',
    '[data-speaker-active="true"]',
    '[data-active-speaker="true"]',
    '[data-testid*="speaker"]',
    '[data-testid*="participant"]',
    '[class*="speaker"]',
    '[class*="participant"]',
    '[class*="talking"]',
    '[class*="speaking"]',
    '[class*="active"]'
  ];
  const labelSelectors = [
    '[data-participant-name]',
    '[data-display-name]',
    '[aria-label]',
    '[title]',
    '[class*="name"]',
    '[class*="title"]',
    '[class*="participant"]'
  ];
  const participantContainer = (el) => el && el.closest
    ? el.closest('[data-participant-id], [data-testid*="participant"], [class*="participant"], [class*="tile"], [class*="speaker"]')
    : null;
  const activeByAttribute = (el) => {{
    if (!el || !el.getAttributeNames) return false;
    return el.getAttributeNames().some((name) => {{
      const lower = name.toLowerCase();
      const value = String(el.getAttribute(name) || '').toLowerCase();
      return (
        (lower.includes('speaker') || lower.includes('active') || lower.includes('talking'))
        && (value === 'true' || value === '1' || activeNeedle.test(value))
      );
    }});
  }};
  const extractLabel = (el) => {{
    if (!el) return '';
    const direct = candidateText(el);
    if (direct && !activeNeedle.test(direct) && !mutedNeedle.test(direct)) return direct;
    const container = participantContainer(el);
    if (!container) return direct;
    for (const selector of labelSelectors) {{
      const candidate = container.querySelector(selector);
      const text = candidateText(candidate || container);
      if (text && !activeNeedle.test(text) && !mutedNeedle.test(text)) return text;
    }}
    return candidateText(container);
  }};
  let last = '';
  let lastAt = 0;
  const emit = async (speakerLabel) => {{
    const label = normalize(speakerLabel);
    if (!label || label === last) return;
    const now = Date.now();
    if (label === last && now - lastAt < 4000) return;
    last = label;
    lastAt = now;
    const invoke = window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
    if (typeof invoke !== 'function') return;
    try {{
      await invoke('yandex_telemost_speaker_timeline_append', {{
        request: {{
          account_id: ACCOUNT_ID,
          conference_id: CONFERENCE_ID,
          speaker_label: label,
          confidence: 0.42,
          observed_at_epoch_ms: now,
          source: 'webview_dom_multi_selector_heuristic'
        }}
      }});
    }} catch (_error) {{}}
  }};
  const scan = () => {{
    const ranked = [];
    for (const selector of selectors) {{
      const nodes = Array.from(document.querySelectorAll(selector)).slice(0, 200);
      for (const el of nodes) {{
        const text = candidateText(el);
        const looksActive = activeNeedle.test(text) || activeByAttribute(el);
        if (!looksActive || mutedNeedle.test(text)) continue;
        const label = extractLabel(el);
        if (!label || label === 'unknown_speaker') continue;
        const score =
          (activeByAttribute(el) ? 3 : 0) +
          (el.getAttribute('data-speaking') === 'true' ? 2 : 0) +
          (text ? 1 : 0) +
          (participantContainer(el) ? 1 : 0);
        ranked.push({{ label, score }});
      }}
    }}
    ranked.sort((left, right) => right.score - left.score);
    if (ranked.length > 0) emit(ranked[0].label);
  }};
  setInterval(scan, 1000);
  new MutationObserver(() => scan()).observe(document.documentElement, {{ childList: true, subtree: true, attributes: true }});
  scan();
}})();
"#
    ))
}

fn prepare_audio_device(
    request: YandexTelemostAudioDevicePrepareRequest,
) -> Result<YandexTelemostAudioDeviceManifest, String> {
    let requested = request
        .device_name
        .unwrap_or_else(|| "hermes_telemost".to_owned());
    #[cfg(target_os = "linux")]
    {
        let sink_arg = format!("sink_name={requested}");
        let sink_properties_arg = format!("sink_properties=device.description={requested}");
        let status = Command::new("pactl")
            .args([
                "load-module",
                "module-null-sink",
                sink_arg.as_str(),
                sink_properties_arg.as_str(),
            ])
            .status();
        let state = match status {
            Ok(status) if status.success() => "created_or_already_available",
            Ok(_) => "pactl_failed_manual_pipewire_or_pulseaudio_setup_required",
            Err(_) => "pactl_not_available_manual_pipewire_or_pulseaudio_setup_required",
        };
        return Ok(YandexTelemostAudioDeviceManifest {
            platform: "linux",
            state,
            input_hint: format!("{requested}.monitor"),
            notes: vec![
                "Route the Telemost WebView output to this sink, then record the monitor source."
                    .to_owned(),
            ],
        });
    }
    #[cfg(target_os = "macos")]
    {
        return Ok(YandexTelemostAudioDeviceManifest {
            platform: "macos",
            state: "external_loopback_driver_required",
            input_hint: requested,
            notes: vec![
                "macOS does not let Hermes silently install a system audio driver.".to_owned(),
                "Install/configure a loopback device such as BlackHole 2ch and set HERMES_TELEMOST_FFMPEG_INPUT if ffmpeg needs a specific device index.".to_owned(),
            ],
        });
    }
    #[cfg(target_os = "windows")]
    {
        return Ok(YandexTelemostAudioDeviceManifest {
            platform: "windows",
            state: "wasapi_loopback_or_virtual_device_required",
            input_hint: requested,
            notes: vec![
                "Use an explicit WASAPI loopback input or a virtual audio cable; set HERMES_TELEMOST_FFMPEG_INPUT for ffmpeg.".to_owned(),
            ],
        });
    }
    #[allow(unreachable_code)]
    Ok(YandexTelemostAudioDeviceManifest {
        platform: "unknown",
        state: "unsupported_platform",
        input_hint: requested,
        notes: vec![
            "No default virtual audio device strategy is available for this platform.".to_owned(),
        ],
    })
}

fn ffmpeg_recording_command(
    audio_input: Option<&str>,
    audio_path: &Path,
) -> Result<Command, String> {
    let ffmpeg_path = std::env::var("HERMES_TELEMOST_FFMPEG_PATH")
        .unwrap_or_else(|_| DEFAULT_FFMPEG_PATH.to_owned());
    let input_override = std::env::var("HERMES_TELEMOST_FFMPEG_INPUT")
        .ok()
        .or_else(|| audio_input.map(str::to_owned));
    let mut command = Command::new(ffmpeg_path);
    command.arg("-y");
    append_ffmpeg_input_args(&mut command, input_override.as_deref())?;
    command.args(["-vn", "-codec:a", "libmp3lame", "-b:a", "128k"]);
    command.arg(audio_path);
    Ok(command)
}

fn append_ffmpeg_input_args(command: &mut Command, input: Option<&str>) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        command.args(["-f", "pulse", "-i", input.unwrap_or(DEFAULT_LINUX_MONITOR)]);
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        command.args([
            "-f",
            "avfoundation",
            "-i",
            input.unwrap_or(":BlackHole 2ch"),
        ]);
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        let device_arg = format!("audio={}", input.unwrap_or("Stereo Mix"));
        command.args(["-f", "dshow", "-i", device_arg.as_str()]);
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("Telemost local recording is unsupported on this platform".to_owned())
}

fn recording_output_dir(
    app: &AppHandle,
    account_id: &str,
    session_id: &str,
) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data dir: {error}"))?;
    Ok(base
        .join("telemost-recordings")
        .join(account_id)
        .join(session_id))
}

fn write_timeline_header(
    path: &Path,
    request: &YandexTelemostRecordingStartRequest,
    session_id: &str,
) -> Result<(), String> {
    append_text_line(path, &format!("# Hermes Yandex Telemost speaker timeline"))?;
    append_text_line(path, &format!("# session_id={session_id}"))?;
    append_text_line(path, &format!("# account_id={}", request.account_id))?;
    append_text_line(
        path,
        &format!(
            "# conference_id={}",
            request.conference_id.as_deref().unwrap_or("")
        ),
    )?;
    append_text_line(
        path,
        "# columns: epoch_ms\tspeaker_label\tevent\tconfidence\tsource",
    )?;
    append_text_line(
        path,
        &format!(
            "{}\tSYSTEM\trecording_start\tconfidence=1.00\tsource=local_recorder",
            now_epoch_ms()
        ),
    )
}

fn append_text_line(path: &Path, line: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to append {}: {error}", path.display()))?;
    writeln!(file, "{line}").map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn companion_window_label(account_id: &str, conference_id: Option<&str>) -> Result<String, String> {
    let account = sanitize_slug(required_slug("account_id", account_id)?);
    if account.is_empty() {
        return Err("account_id must contain at least one slug-safe character".to_owned());
    }
    let conference = conference_id
        .map(sanitize_slug)
        .filter(|value| !value.is_empty());
    Ok(match conference {
        Some(conference) => format!("{WINDOW_LABEL_PREFIX}-{account}-{conference}"),
        None => format!("{WINDOW_LABEL_PREFIX}-{account}"),
    })
}

fn recording_session_id(account_id: &str, conference_id: Option<&str>) -> String {
    let conference = conference_id
        .map(sanitize_slug)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "manual".to_owned());
    format!(
        "{}-{}-{}",
        sanitize_slug(account_id),
        conference,
        now_epoch_ms()
    )
}

fn validate_join_url(value: &str) -> Result<(), String> {
    if !value.starts_with("https://") {
        return Err("Yandex Telemost join URL must be HTTPS".to_owned());
    }
    let host = value
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default();
    if matches!(host, TELEMOST_ALLOWED_HOST_RU | TELEMOST_ALLOWED_HOST_COM) {
        Ok(())
    } else {
        Err(format!(
            "unsupported Yandex Telemost join URL host `{host}`"
        ))
    }
}

fn required_slug<'a>(field: &'static str, value: &'a str) -> Result<&'a str, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(value)
}

fn sanitize_slug(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn sanitize_speaker_label(value: &str) -> String {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let value = value.trim();
    if value.is_empty() {
        "unknown_speaker".to_owned()
    } else {
        value.chars().take(120).collect()
    }
}

fn now_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_telemost_urls() {
        assert!(validate_join_url("https://example.com/room").is_err());
        assert!(validate_join_url("https://telemost.yandex.ru/j/123").is_ok());
    }

    #[test]
    fn labels_are_stable_and_slugged() {
        assert_eq!(
            companion_window_label("Main Account", Some("Conf 1")).unwrap(),
            "yandex-telemost-main-account-conf-1"
        );
    }

    #[test]
    fn speaker_label_is_truncated_and_normalized() {
        assert_eq!(sanitize_speaker_label("  Alice   Smith  "), "Alice Smith");
    }

    #[test]
    fn initialization_script_contains_multi_selector_speaker_heuristics() {
        let script = telemost_initialization_script(
            &YandexTelemostCompanionOpenRequest {
                account_id: "telemost-main".to_owned(),
                join_url: "https://telemost.yandex.ru/j/conf-1".to_owned(),
                conference_id: Some("conf-1".to_owned()),
                display_name: None,
            },
            "yandex-telemost-main-conf-1",
        )
        .expect("initialization script");

        assert!(script.contains("data-active-speaker"));
        assert!(script.contains("data-participant-name"));
        assert!(script.contains("webview_dom_multi_selector_heuristic"));
    }
}
