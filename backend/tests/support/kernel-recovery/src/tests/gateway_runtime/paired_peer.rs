use super::*;

#[test]
fn paired_remote_listener_serves_multiple_tls_peers_until_owner_shutdown() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let (acceptor, connector) = tls_endpoints();
    let listener = runtime
        .block_on(GatewayTlsListenerV1::bind(
            "127.0.0.1:0".parse().expect("loopback address"),
            PairedRemoteProfileV1::new(true, false).expect("remote profile"),
            acceptor,
        ))
        .expect("TLS listener");
    let address = listener.local_address().expect("local address");
    let (shutdown, receiver) = watch::channel(false);
    let server = runtime.spawn(async move {
        listener
            .serve_until_shutdown(GatewayTechnicalRouter::new(true), receiver)
            .await
            .expect("TLS HTTP/2 server")
    });
    runtime.block_on(async move {
        for _ in 0..2 {
            assert_tls_health_peer(address, &connector).await;
        }
        shutdown.send(true).expect("request shutdown");
        server.await.expect("server task");
    });
}

async fn assert_tls_health_peer(address: std::net::SocketAddr, connector: &TlsConnector) {
    let stream = TcpStream::connect(address).await.expect("TCP connection");
    let stream = connector
        .connect(
            ServerName::try_from("localhost")
                .expect("server name")
                .to_owned(),
            stream,
        )
        .await
        .expect("TLS handshake");
    let (mut client, connection) = h2::client::handshake(stream)
        .await
        .expect("HTTP/2 handshake");
    let connection = tokio::spawn(connection);
    let request = Request::builder()
        .method(Method::GET)
        .uri("https://localhost/healthz")
        .body(())
        .expect("HTTP/2 request");
    let (response, _) = client
        .send_request(request, true)
        .expect("send HTTP/2 request");
    assert_eq!(
        response.await.expect("HTTP/2 response").status(),
        StatusCode::OK
    );
    drop(client);
    connection
        .await
        .expect("HTTP/2 connection task")
        .expect("HTTP/2 connection");
}

#[test]
fn paired_remote_listener_refuses_to_accept_after_pre_requested_shutdown() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let (acceptor, _) = tls_endpoints();
    let listener = runtime
        .block_on(GatewayTlsListenerV1::bind(
            "127.0.0.1:0".parse().expect("loopback address"),
            PairedRemoteProfileV1::new(true, false).expect("remote profile"),
            acceptor,
        ))
        .expect("TLS listener");
    let address = listener.local_address().expect("local address");
    let (shutdown, receiver) = watch::channel(true);
    runtime.block_on(async move {
        listener
            .serve_until_shutdown(GatewayTechnicalRouter::new(true), receiver)
            .await
            .expect("listener exits without accepting");
        assert!(
            TcpStream::connect(address).await.is_err(),
            "listener must release the socket when shutdown precedes its loop"
        );
    });
    drop(shutdown);
}

pub(super) fn tls_endpoints() -> (TlsAcceptor, TlsConnector) {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certified = generate_simple_self_signed(vec!["localhost".to_owned()])
        .expect("self-signed test certificate");
    let certificate = certified.cert.der().clone();
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
    let server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], key)
        .expect("server configuration");
    let server = {
        let mut server = server;
        server.alpn_protocols = vec![b"h2".to_vec()];
        server
    };
    let mut roots = RootCertStore::empty();
    roots.add(certificate).expect("test certificate root");
    let client = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let client = {
        let mut client = client;
        client.alpn_protocols = vec![b"h2".to_vec()];
        client
    };
    (
        TlsAcceptor::from(Arc::new(server)),
        TlsConnector::from(Arc::new(client)),
    )
}
