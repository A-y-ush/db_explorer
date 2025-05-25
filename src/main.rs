mod cli;
mod db;
use crate::cli::{Cli, Commands};
use crate::db::{connection::connect_to_db, session::Session};
use clap::Parser;
use log::{error, info};
use rustyline::error::ReadlineError;
use rustyline::{Editor, history::DefaultHistory};
use std::fmt;

#[derive(Debug)]
enum AppError {
    Database(sqlx::Error),
    Readline(ReadlineError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::Readline(err) => write!(f, "Readline error: {}", err),
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

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let startup_args = crate::cli::StartupArgs::parse();
    let pool = connect_to_db(&startup_args.db_url).await?;
    info!("Connected to the DB.");

    let session = Session::start(pool).await?;
    info!("Session initialized.");

    let mut rl = Editor::<(), DefaultHistory>::new()?;
    
    let history_path = ".db_navigator_history";
    if rl.load_history(history_path).is_err() {
        info!("No previous history.");
    }

    loop {
        let readline = rl.readline("dbnav> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                let mut full_args = vec!["dbnav"];
                full_args.extend(line.split_whitespace());
                let args = match Cli::try_parse_from(full_args) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Invalid command: {}", e);
                        continue;
                    }
                };

                match args.command {
                    Commands::Exit => {
                        info!("Exiting...");
                        break;
                    }
                    Commands::ShowSchema => {
                        info!("Showing schema...");
                        session.print_schema();

                    }
                    Commands::ListTables => {
                        let tables = crate::db::introspect::fetch_tables(&session.pool).await?;
                        println!("Tables:");
                        for (i, t) in tables.iter().enumerate() {
                            println!("{}: {}", i + 1, t);
                        }
                    }
                    Commands::Query {
                        table,
                        column,
                        value,
                    } => {
                        println!("Querying table: {}", table);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                error!("Error reading line: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(history_path)?;

    Ok(())
}
