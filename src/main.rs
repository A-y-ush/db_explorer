mod cli;
mod db;
use clap::Parser;
use cli::Args;
use db::{connection::connect_to_db, introspect::fetch_tables,session::Session};
use dotenvy::dotenv;
use log::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    env_logger::init();

    let args = Args::parse();

    let pool = connect_to_db(&args.db_url).await?;
    info!("Connected to the DB.\n");

    let session = Session::start(pool).await?;
    info!("Session initialized.");

    let tables: Vec<String> = fetch_tables(&session.pool).await?;

    for (i, table) in tables.iter().enumerate() {
        println!("[{:>3}] {}", i + 1, table);
    }

    Ok(())
}
