/*
 * Test-only ABI fixture for managed Telegram runtime admission.
 *
 * It proves exact native artifact binding and process composition. It has no
 * audio device, network transport or production media behavior.
 */

#include "../../../src/telegram-call-media-tgcalls/native/bridge.h"

#include <stdlib.h>
#include <string.h>

typedef struct {
    int stopped;
    int state_event_pending;
    int muted;
} HermesTgCallsFixtureSession;

static const char *const HERMES_TGCALLS_VERSIONS[] = {"13.0.0", "14.0.0"};

static int32_t fill_snapshot(
    const HermesTgCallsFixtureSession *session,
    HermesTgCallsSnapshotV1 *snapshot_out) {
    if (session == NULL || snapshot_out == NULL) {
        return HERMES_TGCALLS_INVALID_ARGUMENT_V1;
    }
    snapshot_out->abi_version = HERMES_TGCALLS_ABI_VERSION_V1;
    snapshot_out->state = HERMES_TGCALLS_ESTABLISHED_V1;
    snapshot_out->duration_seconds = session->stopped ? 1u : 0u;
    snapshot_out->connection_id = 7001;
    snapshot_out->failed = 0;
    return HERMES_TGCALLS_OK_V1;
}

uint32_t hermes_tgcalls_abi_version_v1(void) {
    return HERMES_TGCALLS_ABI_VERSION_V1;
}

size_t hermes_tgcalls_version_count_v1(void) {
    return sizeof(HERMES_TGCALLS_VERSIONS) / sizeof(HERMES_TGCALLS_VERSIONS[0]);
}

int32_t hermes_tgcalls_version_at_v1(
    size_t index,
    char *output,
    size_t output_capacity) {
    if (index >= hermes_tgcalls_version_count_v1() || output == NULL) {
        return HERMES_TGCALLS_INVALID_ARGUMENT_V1;
    }
    size_t length = strlen(HERMES_TGCALLS_VERSIONS[index]);
    if (output_capacity <= length) {
        return HERMES_TGCALLS_BUFFER_TOO_SMALL_V1;
    }
    memcpy(output, HERMES_TGCALLS_VERSIONS[index], length + 1);
    return HERMES_TGCALLS_OK_V1;
}

int32_t hermes_tgcalls_max_layer_v1(void) {
    return 92;
}

int32_t hermes_tgcalls_session_create_v1(
    const HermesTgCallsSessionConfigV1 *config,
    void **session_out) {
    if (config == NULL || session_out == NULL
        || config->abi_version != HERMES_TGCALLS_ABI_VERSION_V1
        || config->library_version == NULL
        || (strcmp(config->library_version, "13.0.0") != 0
            && strcmp(config->library_version, "14.0.0") != 0)
        || config->encryption_key == NULL
        || config->encryption_key_length != HERMES_TGCALLS_KEY_BYTES_V1
        || config->servers == NULL
        || config->server_count == 0) {
        return HERMES_TGCALLS_INVALID_ARGUMENT_V1;
    }
    HermesTgCallsFixtureSession *session =
        calloc(1, sizeof(HermesTgCallsFixtureSession));
    if (session == NULL) {
        return HERMES_TGCALLS_NATIVE_FAILURE_V1;
    }
    session->state_event_pending = 1;
    *session_out = session;
    return HERMES_TGCALLS_OK_V1;
}

int32_t hermes_tgcalls_session_receive_signaling_v1(
    void *raw_session,
    const uint8_t *data,
    size_t data_length) {
    HermesTgCallsFixtureSession *session = raw_session;
    if (session == NULL || session->stopped || data == NULL || data_length == 0) {
        return HERMES_TGCALLS_INVALID_ARGUMENT_V1;
    }
    return HERMES_TGCALLS_OK_V1;
}

int32_t hermes_tgcalls_session_set_muted_v1(void *raw_session, uint8_t muted) {
    HermesTgCallsFixtureSession *session = raw_session;
    if (session == NULL || session->stopped) {
        return HERMES_TGCALLS_INVALID_STATE_V1;
    }
    session->muted = muted != 0;
    return HERMES_TGCALLS_OK_V1;
}

int32_t hermes_tgcalls_session_poll_event_v1(
    void *raw_session,
    HermesTgCallsEventV1 *event_out,
    uint8_t *payload_out,
    size_t payload_capacity) {
    HermesTgCallsFixtureSession *session = raw_session;
    (void)payload_out;
    (void)payload_capacity;
    if (session == NULL || event_out == NULL) {
        return HERMES_TGCALLS_INVALID_ARGUMENT_V1;
    }
    if (!session->state_event_pending) {
        return HERMES_TGCALLS_OK_V1;
    }
    session->state_event_pending = 0;
    event_out->abi_version = HERMES_TGCALLS_ABI_VERSION_V1;
    event_out->kind = HERMES_TGCALLS_STATE_EVENT_V1;
    event_out->state = HERMES_TGCALLS_ESTABLISHED_V1;
    event_out->payload_length = 0;
    return HERMES_TGCALLS_EVENT_V1;
}

int32_t hermes_tgcalls_session_snapshot_v1(
    void *raw_session,
    HermesTgCallsSnapshotV1 *snapshot_out) {
    return fill_snapshot(raw_session, snapshot_out);
}

int32_t hermes_tgcalls_session_stop_v1(
    void *raw_session,
    HermesTgCallsSnapshotV1 *snapshot_out) {
    HermesTgCallsFixtureSession *session = raw_session;
    if (session == NULL) {
        return HERMES_TGCALLS_INVALID_ARGUMENT_V1;
    }
    session->stopped = 1;
    return fill_snapshot(session, snapshot_out);
}

int32_t hermes_tgcalls_session_destroy_v1(void *raw_session) {
    HermesTgCallsFixtureSession *session = raw_session;
    if (session == NULL || !session->stopped) {
        return HERMES_TGCALLS_INVALID_STATE_V1;
    }
    free(session);
    return HERMES_TGCALLS_OK_V1;
}
