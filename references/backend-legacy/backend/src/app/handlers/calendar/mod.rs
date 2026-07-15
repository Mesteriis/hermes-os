// ADR-0073: calendar handlers are split by documented Calendar domain responsibilities.
pub(crate) mod accounts;
pub(crate) mod analytics;
pub(crate) mod brain;
pub(crate) mod events;
pub(crate) mod health;
pub(crate) mod intelligence;
pub(crate) mod meetings;
pub(crate) mod reminders;
pub(crate) mod rules;
pub(crate) mod scheduling;
pub(crate) mod search;
pub(crate) mod sync;

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::domains::calendar::brain::CalendarBrainService;
use crate::domains::calendar::command_service::CalendarCommandService;
use crate::domains::calendar::core::agendas::EventAgendaStore;
use crate::domains::calendar::core::checklists::EventChecklistStore;
use crate::domains::calendar::core::context_packs::{ContextPackInput, EventContextPackStore};
use crate::domains::calendar::core::participants::EventParticipantStore;
use crate::domains::calendar::core::relations::EventRelationStore;
use crate::domains::calendar::events::event_store::CalendarEventStore;
use crate::domains::calendar::events::models::{
    CalendarAccountUpdate, CalendarEventUpdate, NewCalendarEvent,
};
use crate::domains::calendar::events::queries::CalendarEventListQuery;
use crate::domains::calendar::health::CalendarWatchtowerService;
use crate::domains::calendar::intelligence::CalendarIntelligenceService;
use crate::domains::calendar::meetings::notes::MeetingNoteStore;
use crate::domains::calendar::meetings::outcomes::MeetingOutcomeStore;
use crate::domains::calendar::meetings::recordings::EventRecordingStore;
use crate::domains::calendar::meetings::transcripts::EventTranscriptStore;
use crate::domains::calendar::reminders::CalendarReminderStore;
use crate::domains::calendar::rules::{CalendarRuleStore, RuleUpdate};
use crate::domains::calendar::scheduling::{
    DeadlineStore, FocusBlockStore, SmartSchedulingService,
};
use crate::domains::calendar::sync::{export_event_ics, export_event_md};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::calendar_meeting_outcomes::CalendarMeetingOutcomeApplicationService;
