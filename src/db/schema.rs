use std::collections::HashMap;
use sqlx::PgPool;
use super::introspect::{fetch_tables, fetch_all_columns, fetch_all_foreign_keys};

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug,Clone)]
pub struct ForeignKey {
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
}

#[derive(Debug)]
pub struct Schema {
    pub tables: HashMap<String, Table>,
}

impl Schema {
    pub async fn load(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let tables = fetch_tables(pool).await?;
        let columns_map = fetch_all_columns(pool).await?;
        let fk_map = fetch_all_foreign_keys(pool).await?;

        let mut schema = HashMap::new();
        for table_name in tables {
            let columns = columns_map.get(&table_name).cloned().unwrap_or_default();
            let foreign_keys = fk_map.get(&table_name).cloned().unwrap_or_default();

            schema.insert(
                table_name.clone(),
                Table {
                    name: table_name,
                    columns,
                    foreign_keys,
                },
            );
        }

        Ok(Schema { tables: schema })
    }
}
