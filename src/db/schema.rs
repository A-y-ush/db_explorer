use std::collections::{HashMap, VecDeque, HashSet};
use sqlx::PgPool;
use super::introspect::{fetch_tables, fetch_all_columns, fetch_all_foreign_keys};

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub foreign_keys: Vec<ForeignKey>,
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

    pub fn find_join_path(&self, from: &str, to: &str) -> Result<Vec<ForeignKey>, &'static str> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent_map: HashMap<String, Option<(String, ForeignKey)>> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());
        parent_map.insert(from.to_string(), None);

        while let Some(current) = queue.pop_front() {
            if current == to {
                break;
            }

            if let Some(table) = self.tables.get(&current) {
                for fk in &table.foreign_keys {
                    if !visited.contains(&fk.to_table) {
                        visited.insert(fk.to_table.clone());
                        parent_map.insert(fk.to_table.clone(), Some((current.clone(), fk.clone())));
                        queue.push_back(fk.to_table.clone());
                    }
                }

                for (table_name, t) in &self.tables {
                    for fk in &t.foreign_keys {
                        if fk.to_table == current && !visited.contains(&fk.from_table) {
                            visited.insert(fk.from_table.clone());
                            parent_map.insert(fk.from_table.clone(), Some((current.clone(), fk.clone())));
                            queue.push_back(fk.from_table.clone());
                        }
                    }
                }
            }
        }

        if !parent_map.contains_key(to) {
            return Err("No join path found");
        }

        let mut path = Vec::new();
        let mut current = to.to_string();

        while let Some(Some((prev, fk))) = parent_map.get(&current) {
            path.push(fk.clone());
            current = prev.clone();
        }

        path.reverse();
        Ok(path)
    }

    pub fn generate_sql(
        &self,
        target_table: &str,
        target_column: &str,
        condition_table: &str,
        condition_column: &str,
        condition_value: &str,
    ) -> Result<String, &'static str> {
        let path = self.find_join_path(condition_table, target_table)?;
        let mut query = format!("SELECT {}.{} FROM {}", target_table, target_column, condition_table);
        let mut last_table = condition_table.to_string();

        for fk in path {
            query.push_str(&format!(
                " JOIN {} ON {}.{} = {}.{}",
                if fk.from_table == last_table { &fk.to_table } else { &fk.from_table },
                fk.from_table, fk.from_column,
                fk.to_table, fk.to_column
            ));
            last_table = if fk.from_table == last_table { fk.to_table.clone() } else { fk.from_table.clone() };
        }

        query.push_str(&format!(" WHERE {}.{} = '{}'", condition_table, condition_column, condition_value));
        Ok(query)
    }
}
