use std::collections::HashMap;
use sqlx::{PgPool, Row};
use super::introspect::{fetch_tables, fetch_columns, fetch_foreign_keys};

#[derive(Debug)]
pub struct Table{
    pub name:String,
    pub columns: Vec<String>,
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug)]
pub struct ForeignKey{
    pub from_table:String,
    pub from_column:String,
    pub to_table:String,
    pub to_column:String,
}

#[derive(Debug)]
pub struct Schema{
    pub tables: HashMap<String,Table>,
}

impl Schema{
    pub async fn load(pool: &PgPool)-> Result<Self,sqlx::Error>{
        let tables = fetch_tables(pool).await?;
        let mut schema = HashMap::new();

        for table_name in tables{
            let columns = fetch_columns(pool,&table_name).await?;
            let foreign_keys = fetch_foreign_keys(pool,&table_name).await?;

            let table = Table{
                name: table_name.clone(),
                columns,
                foreign_keys,
            };

            schema.insert(table_name,table);
        }

        Ok(Schema{tables:schema})
    }
}
