use hermes_hub_backend::platform::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    hermes_hub_backend::app::init_tracing();

    let config = AppConfig::from_env()?;
    hermes_hub_backend::app::run(config).await?;

    Ok(())
}
