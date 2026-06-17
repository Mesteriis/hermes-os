use hermes_hub_backend::app::init_tracing;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = AppConfig::from_env()?;
    let database_url = config
        .database_url()
        .ok_or("DATABASE_URL is required for migrations")?;

    Database::connect(Some(database_url)).await?;
    println!("Hermes backend migrations and startup repairs completed.");

    Ok(())
}
