use sqlx::{PgPool, Row};
use crate::db::schema::Schema;

pub struct Session {
    pub pool: PgPool,
    pub schema: Schema,
}

impl Session {
    pub async fn start(pool: PgPool) -> Result<Self, sqlx::Error> {
        let schema = Schema::load(&pool).await?;
        Ok(Self { pool, schema })
    }

    pub fn show_schema(&self) {
        for (table_name, table) in &self.schema.tables {
            println!("\nTable: {}", table_name);
            println!("  Columns:");
            for col in &table.columns {
                println!("    - {}", col);
            }
            if !table.foreign_keys.is_empty() {
                println!("  Foreign Keys:");
                for fk in &table.foreign_keys {
                    println!(
                        "    - {}.{} → {}.{}",
                        fk.from_table, fk.from_column, fk.to_table, fk.to_column
                    );
                }
            }
        }
    }

    pub async fn query(
        &self,
        target_table: &str,
        target_column: &str,
        condition: &str,
        value: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let (cond_table, cond_col) = condition.split_once('.')
            .ok_or("Invalid --where format: expected 'table.column'")?;

        let sql = self.schema.generate_sql(
            target_table,
            target_column,
            cond_table,
            cond_col,
            value,
        ).map_err(|e| format!("Failed to generate SQL: {}", e))?;

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await
            .map_err(|e| format!("Database query failed: {}", e))?;
        
        let mut results = Vec::new();

        for row in rows {
            // Handle different column types more robustly
            let val = match row.try_get::<String, _>(target_column) {
                Ok(s) => s,
                Err(_) => {
                    // Try other common types and convert to string
                    if let Ok(i) = row.try_get::<i32, _>(target_column) {
                        i.to_string()
                    } else if let Ok(i) = row.try_get::<i64, _>(target_column) {
                        i.to_string()
                    } else if let Ok(f) = row.try_get::<f64, _>(target_column) {
                        f.to_string()
                    } else if let Ok(b) = row.try_get::<bool, _>(target_column) {
                        b.to_string()
                    } else {
                        // If all else fails, try to get as Option<String> for nullable columns
                        match row.try_get::<Option<String>, _>(target_column) {
                            Ok(Some(s)) => s,
                            Ok(None) => "NULL".to_string(),
                            Err(e) => return Err(format!("Cannot convert column '{}' to string: {}", target_column, e).into()),
                        }
                    }
                }
            };
            results.push(val);
        }

        Ok(results)
    }
}