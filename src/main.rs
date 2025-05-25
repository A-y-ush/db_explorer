mod cli;
mod db;

use crate::cli::{Cli, Commands};
use crate::db::{connection::connect_to_db, session::Session};
use clap::Parser;
use log::{error, info};
use rustyline::{Editor, error::ReadlineError, history::DefaultHistory};
use std::fmt;

#[derive(Debug)]
enum AppError {
    Database(sqlx::Error),
    Readline(ReadlineError),
    Query(Box<dyn std::error::Error + Send + Sync>),
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::Readline(err) => write!(f, "Readline error: {}", err),
            AppError::Query(err) => write!(f, "Query error: {}", err),
            AppError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<ReadlineError> for AppError {
    fn from(err: ReadlineError) -> Self {
        AppError::Readline(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AppError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AppError::Query(err)
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let startup_args = crate::cli::StartupArgs::parse();
    let pool = connect_to_db(&startup_args.db_url).await?;
    info!("Connected to DB.");

    let session = Session::start(pool).await?;
    info!("Session initialized.");

    let mut rl = Editor::<(), DefaultHistory>::new()
        .map_err(|e| AppError::Other(format!("Failed to create readline editor: {}", e)))?;
    let history_path = ".dbnav_history";

    if let Err(e) = rl.load_history(history_path) {
        info!("No previous history found: {}", e);
    }

    println!("Welcome to dbnav! Type 'exit' to quit or use --help for available commands.");

    loop {
        let readline = rl.readline("dbnav> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if let Err(e) = rl.add_history_entry(line) {
                    error!("Failed to add history entry: {}", e);
                    // Continue execution - this is not a fatal error
                }

                let mut full_args = vec!["dbnav"];
                full_args.extend(line.split_whitespace());
                
                let args = match Cli::try_parse_from(full_args) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Invalid command: {}", e);
                        continue;
                    }
                };

                if let Err(e) = handle_command(&session, args.command).await {
                    error!("Command failed: {}", e);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                error!("Terminal error: {}", err);
                return Err(AppError::Readline(err));
            }
        }
    }

    if let Err(e) = rl.save_history(history_path) {
        error!("Failed to save history: {}", e);
        // Don't fail the program just because we couldn't save history
    }

    Ok(())
}

async fn handle_command(session: &Session, command: Commands) -> Result<(), AppError> {
    match command {
        Commands::Exit => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        Commands::ShowSchema => {
            session.show_schema();
        }
        Commands::ListTables => {
            let tables = db::introspect::fetch_tables(&session.pool).await?;
            if tables.is_empty() {
                println!("No tables found.");
            } else {
                println!("Tables:");
                for table in tables {
                    println!("  - {}", table);
                }
            }
        }
        Commands::Query {
            table,
            column,
            value,
            r#where,
        } => {
            // Handle optional fields - these should be required for the query to work
            let column_str = column.as_deref().ok_or_else(|| AppError::Other("Column name is required (use -c or --column)".to_string()))?;
            let value_str = value.as_deref().ok_or_else(|| AppError::Other("Value is required (use -v or --value)".to_string()))?;
            
            println!("Querying table '{}' for column '{}' where {} = '{}'", table, column_str, r#where, value_str);
            
            let rows = session.query(&table, column_str, &r#where, value_str).await?;
            if rows.is_empty() {
                println!("No results found.");
            } else {
                println!("Results ({} found):", rows.len());
                for (i, row) in rows.iter().enumerate() {
                    println!("  {}: {}", i + 1, row);
                }
            }
        }
    }
    Ok(())
}