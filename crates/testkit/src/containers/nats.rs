use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::time::{Duration, Instant, sleep};

const NATS_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const NATS_CONNECT_RETRY_DELAY: Duration = Duration::from_millis(250);

pub struct NatsContainer {
    _container: ContainerAsync<GenericImage>,
    host_port: u16,
}

impl NatsContainer {
    pub async fn start() -> Self {
        let container = GenericImage::new("nats", "2.11-alpine")
            .with_exposed_port(4222.tcp())
            .with_cmd(vec!["-js", "-sd", "/data"])
            .start()
            .await
            .expect("failed to start NATS container");

        let host_port = container
            .get_host_port_ipv4(4222)
            .await
            .expect("failed to resolve NATS container port");

        wait_for_nats_port(host_port).await;

        Self {
            _container: container,
            host_port,
        }
    }

    pub fn server_url(&self) -> String {
        format!("nats://127.0.0.1:{}", self.host_port)
    }
}

async fn wait_for_nats_port(host_port: u16) {
    let deadline = Instant::now() + NATS_CONNECT_TIMEOUT;
    let server_url = format!("nats://127.0.0.1:{host_port}");

    loop {
        match async_nats::connect(&server_url).await {
            Ok(client) => {
                client.flush().await.expect("flush NATS test client");
                return;
            }
            Err(_) if Instant::now() < deadline => {
                sleep(NATS_CONNECT_RETRY_DELAY).await;
            }
            Err(error) => panic!("failed to connect to NATS test container: {error}"),
        }
    }
}
