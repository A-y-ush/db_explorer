use sqlx::PgPool;
use std::collections::HashMap;
use crate::db::schema::ForeignKey;
use crate::db::introspect::{fetch_all_columns, fetch_all_foreign_keys, fetch_tables};

pub struct Session {
    pub pool: PgPool,
    pub tables: Vec<String>,
    pub columns: HashMap<String, Vec<String>>,
    pub foreign_keys: HashMap<String, Vec<ForeignKey>>,
}

impl Session {
    pub async fn start(pool: PgPool) -> Result<Self, sqlx::Error> {
        let tables = fetch_tables(&pool).await?;
        let columns = fetch_all_columns(&pool).await?;
        let foreign_keys = fetch_all_foreign_keys(&pool).await?;

        Ok(Self {
            pool,
            tables,
            columns,
            foreign_keys,
        })
    }

    pub fn print_schema(&self) {
        println!("Schema Overview:");
        for table in &self.tables {
            println!("Table: {}", table);
            if let Some(cols) = self.columns.get(table) {
                for col in cols {
                    println!("  - Column: {}", col);
                }
            }
            if let Some(fks) = self.foreign_keys.get(table) {
                for fk in fks {
                    println!(
                        "  - Foreign Key: {}.{} → {}.{}",
                        fk.from_table, fk.from_column, fk.to_table, fk.to_column
                    );
                }
            }
        }
    }
}
