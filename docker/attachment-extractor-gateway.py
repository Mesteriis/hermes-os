#!/usr/bin/env python3
"""Loopback-only bridge to the network-isolated attachment extractor."""

import socket
from socketserver import BaseRequestHandler, ThreadingTCPServer


MAX_REQUEST_BYTES = 8 * 1024
MAX_RESPONSE_BYTES = 3 * 1024 * 1024
UPSTREAM = ("attachment-extractor", 8788)


def receive_request(connection: socket.socket) -> bytes:
    chunks = []
    size = 0
    while True:
        chunk = connection.recv(min(4 * 1024, MAX_REQUEST_BYTES + 1 - size))
        if not chunk:
            return b"".join(chunks)
        chunks.append(chunk)
        size += len(chunk)
        if size > MAX_REQUEST_BYTES or b"\n" in chunk:
            return b"".join(chunks)


class GatewayHandler(BaseRequestHandler):
    def handle(self) -> None:
        self.request.settimeout(25)
        request = receive_request(self.request)
        if not request or len(request) > MAX_REQUEST_BYTES:
            return
        try:
            with socket.create_connection(UPSTREAM, timeout=5) as upstream:
                upstream.settimeout(25)
                upstream.sendall(request)
                chunks = []
                response_size = 0
                while chunk := upstream.recv(min(64 * 1024, MAX_RESPONSE_BYTES + 1 - response_size)):
                    chunks.append(chunk)
                    response_size += len(chunk)
                    if response_size > MAX_RESPONSE_BYTES:
                        return
                response = b"".join(chunks)
        except OSError:
            return
        if response and len(response) <= MAX_RESPONSE_BYTES:
            self.request.sendall(response)


class Gateway(ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


if __name__ == "__main__":
    with Gateway(("0.0.0.0", 8788), GatewayHandler) as server:
        server.serve_forever()
