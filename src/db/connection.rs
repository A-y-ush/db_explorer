use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
pub async fn connect_to_db(url: &str)-> Result<PgPool, sqlx::Error>{
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    Ok(pool)
}