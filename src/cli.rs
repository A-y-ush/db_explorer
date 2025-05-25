use clap:: {Parser,Subcommand,CommandFactory};

// CLI Arguments
#[derive(Parser, Debug)]
pub struct StartupArgs{
    // DB Connection String
    #[arg(short,long)]
    pub db_url: String,
}

#[derive(Parser, Debug)]
#[command(name="DB Navigator")]
#[command(about = "CLI tool to query relational data")]
pub struct Cli{
    #[command(subcommand)]
    pub command:Commands,
}

#[derive(Subcommand,Debug)]
pub enum Commands{
   
    ShowSchema,
    ListTables,
    Query {
        table: String,
        #[arg(short, long)]
        column: Option<String>,
        #[arg(short, long)]
        value: Option<String>,
        #[arg(long)]
        r#where: String,
    },
    Exit,
}




