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

    let table_names = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("table_name").ok())
        .collect();

    Ok(table_names)
}

pub async fn fetch_columns(pool: &PgPool, table_name: &str) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT column_name
        FROM information_schema.columns
        WHERE table_name = $1
        "#,
    )
    .bind(table_name)
    .fetch_all(pool)
    .await?;

    let column_names:Vec<String> = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("column_name").ok())
        .collect();

    Ok(column_names)
}

pub async fn fetch_foreign_keys(pool:&PgPool,table:&str)-> Result<Vec<ForeignKey>,sqlx::Error>{
    let rows = sqlx::query!(
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
            constraint_type = 'FOREIGN KEY' AND 
            kcu.table_name = $1
        "#,
        table   
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|r| {
            Some(ForeignKey {
                from_table: r.from_table?,
                from_column: r.from_column?,
                to_table: r.to_table?,
                to_column: r.to_column?,
            })
        })
        .collect())
}
