use std::fs;
use std::env;

use anyhow::Result;

use migration_checker::DatabaseConfig;

#[async_std::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    args.next();

    let path = match args.next() { 
        Some(arg) => arg,
        None => panic!("Migrations directory not supplied as first argument")
    };

    let config = match fs::read_to_string("config.json") {
        Ok(f) => f,
        Err(_) => panic!("Could not find config.json")
    };

    let app_config: DatabaseConfig = match serde_json::from_str(&config) {
        Ok(c) => c,
        Err(e) => panic!("Failed to deserialize config.json: {:?}", e)
    };

    migration_checker::find_missing_migrations(path, app_config).await?;

    Ok(())
}
