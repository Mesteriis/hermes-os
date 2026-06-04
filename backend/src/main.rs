use hermes_hub_backend::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    hermes_hub_backend::init_tracing();

    let config = AppConfig::from_env()?;
    hermes_hub_backend::run(config).await?;

    Ok(())
}
