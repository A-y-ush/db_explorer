use crate::db::{schema::Schema};
use sqlx::PgPool;
use log::info;

pub struct Session{
    pub pool: PgPool,
    pub schema:Schema,
}

impl Session{
    pub async fn start(pool:PgPool)-> Result<Self,sqlx::Error>{
        info!("Session created with schema...");
        let schema = Schema::load(&pool).await?;
        Ok(Self{
            pool,
            schema,
        })
    }
}