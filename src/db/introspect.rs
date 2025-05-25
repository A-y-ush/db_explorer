use std::collections::HashMap;
use sqlx::{PgPool, Row};
use super::schema::ForeignKey;

pub async fn fetch_tables(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public'
        ORDER BY table_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("table_name").ok())
        .collect())
}

pub async fn fetch_all_columns(pool: &PgPool) -> Result<HashMap<String, Vec<String>>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT table_name, column_name
        FROM information_schema.columns
        WHERE table_schema = 'public'
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut map:HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let table = row.try_get::<String, _>("table_name")?;
        let column = row.try_get::<String, _>("column_name")?;
        map.entry(table).or_default().push(column);
    }
    Ok(map)
}

pub async fn fetch_all_foreign_keys(pool: &PgPool) -> Result<HashMap<String, Vec<ForeignKey>>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            kcu.table_name AS from_table,
            kcu.column_name AS from_column,
            ccu.table_name AS to_table,
            ccu.column_name AS to_column
        FROM 
            information_schema.table_constraints AS tc
        JOIN information_schema.key_column_usage AS kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage AS ccu
            ON ccu.constraint_name = tc.constraint_name
        WHERE 
            tc.constraint_type = 'FOREIGN KEY'
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut map:HashMap<String, Vec<ForeignKey>> = HashMap::new();
    for row in rows {
        let fk = ForeignKey {
            from_table: row.try_get("from_table")?,
            from_column: row.try_get("from_column")?,
            to_table: row.try_get("to_table")?,
            to_column: row.try_get("to_column")?,
        };
        map.entry(fk.from_table.clone()).or_default().push(fk);
    }

    Ok(map)
}
