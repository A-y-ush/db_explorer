use clap:: Parser;

// CLI Arguments
#[derive(Parser, Debug)]
#[command(name="DB Navigator")]
#[command(about = "CLI tool to query relational data")]
pub struct Args{
    // DB Connection String
    #[arg(short,long)]
    pub db_url: String,
}
