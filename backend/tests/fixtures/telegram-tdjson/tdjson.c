#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define HERMES_QUEUE_CAPACITY 32
#define HERMES_PAYLOAD_CAPACITY 4096

typedef struct {
    char queue[HERMES_QUEUE_CAPACITY][HERMES_PAYLOAD_CAPACITY];
    size_t head;
    size_t tail;
    char current[HERMES_PAYLOAD_CAPACITY];
} HermesTdJsonClient;

static int enqueue(HermesTdJsonClient *client, const char *payload) {
    size_t next = (client->tail + 1) % HERMES_QUEUE_CAPACITY;
    if (next == client->head || strlen(payload) >= HERMES_PAYLOAD_CAPACITY) {
        return 0;
    }
    strcpy(client->queue[client->tail], payload);
    client->tail = next;
    return 1;
}

static int extract_extra(const char *request, char *extra, size_t capacity) {
    const char *key = strstr(request, "\"@extra\"");
    if (key == NULL) {
        return 0;
    }
    const char *separator = strchr(key, ':');
    if (separator == NULL) {
        return 0;
    }
    const char *start = strchr(separator, '"');
    if (start == NULL) {
        return 0;
    }
    start += 1;
    const char *end = strchr(start, '"');
    if (end == NULL) {
        return 0;
    }
    size_t length = (size_t)(end - start);
    if (length == 0 || length >= capacity) {
        return 0;
    }
    memcpy(extra, start, length);
    extra[length] = '\0';
    return 1;
}

void *td_json_client_create(void) {
    HermesTdJsonClient *client = calloc(1, sizeof(HermesTdJsonClient));
    if (client == NULL) {
        return NULL;
    }
    enqueue(
        client,
        "{\"@type\":\"updateAuthorizationState\",\"authorization_state\":{\"@type\":\"authorizationStateReady\"}}"
    );
    enqueue(
        client,
        "{\"@type\":\"updateNewMessage\",\"message\":{\"id\":7001,\"chat_id\":9001,\"sender_id\":{\"@type\":\"messageSenderUser\",\"user_id\":42},\"is_outgoing\":false,\"date\":1783024000,\"content\":{\"@type\":\"messageText\",\"text\":{\"@type\":\"formattedText\",\"text\":\"managed Telegram evidence\"}}}}"
    );
    return client;
}

void td_json_client_send(void *raw_client, const char *request) {
    HermesTdJsonClient *client = raw_client;
    char extra[512];
    char response[HERMES_PAYLOAD_CAPACITY];
    if (client == NULL || request == NULL ||
        !extract_extra(request, extra, sizeof(extra))) {
        return;
    }
    const char *format = strstr(request, "\"@type\":\"sendMessage\"") == NULL
        ? "{\"@type\":\"ok\",\"@extra\":\"%s\"}"
        : "{\"@type\":\"message\",\"id\":8001,\"@extra\":\"%s\"}";
    int written = snprintf(response, sizeof(response), format, extra);
    if (written > 0 && (size_t)written < sizeof(response)) {
        enqueue(client, response);
    }
    if (strstr(request, "outage replay trigger") != NULL) {
        enqueue(
            client,
            "{\"@type\":\"updateNewMessage\",\"message\":{\"id\":7002,\"chat_id\":9001,\"sender_id\":{\"@type\":\"messageSenderUser\",\"user_id\":42},\"is_outgoing\":false,\"date\":1783024001,\"content\":{\"@type\":\"messageText\",\"text\":{\"@type\":\"formattedText\",\"text\":\"managed Telegram outage replay evidence\"}}}}"
        );
    }
}

const char *td_json_client_receive(void *raw_client, double timeout) {
    HermesTdJsonClient *client = raw_client;
    if (client == NULL) {
        return NULL;
    }
    if (client->head == client->tail) {
        if (timeout > 0.0) {
            double bounded = timeout > 0.05 ? 0.05 : timeout;
            usleep((useconds_t)(bounded * 1000000.0));
        }
        return NULL;
    }
    strcpy(client->current, client->queue[client->head]);
    client->head = (client->head + 1) % HERMES_QUEUE_CAPACITY;
    return client->current;
}

const char *td_json_client_execute(void *raw_client, const char *request) {
    (void)raw_client;
    (void)request;
    return "{\"@type\":\"ok\"}";
}

void td_json_client_destroy(void *raw_client) {
    free(raw_client);
}
