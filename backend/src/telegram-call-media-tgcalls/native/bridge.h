#ifndef HERMES_TELEGRAM_TGCALLS_BRIDGE_H
#define HERMES_TELEGRAM_TGCALLS_BRIDGE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define HERMES_TGCALLS_ABI_VERSION_V1 1u
#define HERMES_TGCALLS_KEY_BYTES_V1 256u
#define HERMES_TGCALLS_PEER_TAG_BYTES_V1 16u

#if defined(__GNUC__) || defined(__clang__)
#define HERMES_TGCALLS_EXPORT __attribute__((visibility("default")))
#else
#define HERMES_TGCALLS_EXPORT
#endif

enum HermesTgCallsResultV1 {
    HERMES_TGCALLS_OK_V1 = 0,
    HERMES_TGCALLS_EVENT_V1 = 1,
    HERMES_TGCALLS_INVALID_ARGUMENT_V1 = -1,
    HERMES_TGCALLS_UNSUPPORTED_VERSION_V1 = -2,
    HERMES_TGCALLS_INVALID_STATE_V1 = -3,
    HERMES_TGCALLS_QUEUE_OVERFLOW_V1 = -4,
    HERMES_TGCALLS_NATIVE_FAILURE_V1 = -5,
    HERMES_TGCALLS_BUFFER_TOO_SMALL_V1 = -6,
};

enum HermesTgCallsServerKindV1 {
    HERMES_TGCALLS_TELEGRAM_REFLECTOR_V1 = 1,
    HERMES_TGCALLS_WEBRTC_V1 = 2,
};

enum HermesTgCallsMediaStateV1 {
    HERMES_TGCALLS_CONNECTING_V1 = 1,
    HERMES_TGCALLS_ESTABLISHED_V1 = 2,
    HERMES_TGCALLS_RECONNECTING_V1 = 3,
    HERMES_TGCALLS_FAILED_V1 = 4,
};

enum HermesTgCallsEventKindV1 {
    HERMES_TGCALLS_STATE_EVENT_V1 = 1,
    HERMES_TGCALLS_SIGNALING_EVENT_V1 = 2,
};

typedef struct HermesTgCallsServerV1 {
    uint32_t abi_version;
    uint32_t kind;
    uint8_t reflector_id;
    uint8_t is_tcp;
    uint8_t supports_stun;
    uint8_t supports_turn;
    uint16_t port;
    const char *host;
    const char *username;
    const char *password;
    uint8_t peer_tag[HERMES_TGCALLS_PEER_TAG_BYTES_V1];
} HermesTgCallsServerV1;

typedef struct HermesTgCallsSessionConfigV1 {
    uint32_t abi_version;
    const char *library_version;
    double initialization_timeout_seconds;
    double receive_timeout_seconds;
    uint8_t enable_p2p;
    uint8_t allow_tcp;
    uint8_t is_outgoing;
    const char *call_config;
    const char *custom_parameters;
    const uint8_t *encryption_key;
    size_t encryption_key_length;
    const HermesTgCallsServerV1 *servers;
    size_t server_count;
    const char *input_device_id;
    const char *output_device_id;
} HermesTgCallsSessionConfigV1;

typedef struct HermesTgCallsEventV1 {
    uint32_t abi_version;
    uint32_t kind;
    uint32_t state;
    size_t payload_length;
} HermesTgCallsEventV1;

typedef struct HermesTgCallsSnapshotV1 {
    uint32_t abi_version;
    uint32_t state;
    uint32_t duration_seconds;
    int64_t connection_id;
    uint8_t failed;
} HermesTgCallsSnapshotV1;

HERMES_TGCALLS_EXPORT uint32_t hermes_tgcalls_abi_version_v1(void);
HERMES_TGCALLS_EXPORT size_t hermes_tgcalls_version_count_v1(void);
HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_version_at_v1(
    size_t index,
    char *output,
    size_t output_capacity);
HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_max_layer_v1(void);

HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_session_create_v1(
    const HermesTgCallsSessionConfigV1 *config,
    void **session_out);
HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_session_receive_signaling_v1(
    void *session,
    const uint8_t *data,
    size_t data_length);
HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_session_set_muted_v1(
    void *session,
    uint8_t muted);
HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_session_poll_event_v1(
    void *session,
    HermesTgCallsEventV1 *event_out,
    uint8_t *payload_out,
    size_t payload_capacity);
HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_session_snapshot_v1(
    void *session,
    HermesTgCallsSnapshotV1 *snapshot_out);
HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_session_stop_v1(
    void *session,
    HermesTgCallsSnapshotV1 *snapshot_out);
HERMES_TGCALLS_EXPORT int32_t hermes_tgcalls_session_destroy_v1(void *session);

#ifdef __cplusplus
}
#endif

#endif
