use std::fs;

use anyhow::Result;
use async_std::net::TcpStream;
use tiberius::{Client, Config, AuthMethod, error::Error};
use tiberius::SqlBrowser;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub server_name: String,
    pub instance_name: String,
    pub port: u16,
    pub db_name: String,
    pub user: String,
    pub password: String
}

pub async fn find_missing_migrations(path: String, app_config: DatabaseConfig) -> Result<()> {
    let mut client = get_client_windows(&app_config).await?;
    
    // select db
    let select_db_query = format!("USE {}", app_config.db_name);
    client.execute(select_db_query, &[]).await?;

    // query applied migrations
    let stream = client.simple_query("SELECT MigrationId from __EFMigrationsHistory").await?;
    let rows = stream.into_first_result().await?;
    
    // map results to a new vector
    let mut applied_migrations: Vec<String> = vec!();

    for row in rows.into_iter() {
        let value: Option<&str> = row.get(0);
        match value {
            Some(v) => applied_migrations.push(v.to_string()),
            None => panic!("Expected row to contain a value"),
        }
    }

    let migration_files = get_migration_files(&path[..])?;

    println!("The following migrations have not been applied:");

    for file in migration_files {
        if !applied_migrations.contains(&file) {
            println!("{}", file);
        }     
    }

    Ok(())
}

async fn get_client_windows(app_config: &DatabaseConfig) -> Result<Client<TcpStream>> {
    let mut config = Config::new();

    config.trust_cert();
    config.host("127.0.0.1");
    config.instance_name(&app_config.instance_name);
    config.port(1434);
    config.authentication(AuthMethod::Integrated);

    let stream = TcpStream::connect_named(&config).await;

    match stream {
        Ok(s) => {
            let client = Client::connect(config, s).await?;
            return Ok(client);
        },
        Err(e) => panic!("Failed to connect to sql server: {:?}", e),
    }
}

fn get_migration_files(path: &str) -> Result<Vec<String>> {
    let migration_files = fs::read_dir(path)?;

    let result: Vec<String> = migration_files
        .filter(|x| !x.as_ref().unwrap().path().is_dir())
        .map(|x| x.unwrap().path().file_stem().unwrap().to_str().unwrap().to_string() )
        .filter(|f| !f.contains(".Designer") && !f.contains("ModelSnapshot"))
        .collect();

    Ok(result)
}