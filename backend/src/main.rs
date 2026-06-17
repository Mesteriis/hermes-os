use hermes_hub_backend::platform::config::AppConfig;
use tracing::Instrument;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    hermes_hub_backend::app::init_tracing();
    let flow_id = std::env::var("HERMES_FLOW_ID").unwrap_or_else(|_| "unknown".to_owned());
    let runtime_span = tracing::info_span!("hermes_runtime", flow_id = %flow_id);

    async move {
        let config = AppConfig::from_env()?;
        hermes_hub_backend::app::run(config).await?;
        Ok(())
    }
    .instrument(runtime_span)
    .await
}
