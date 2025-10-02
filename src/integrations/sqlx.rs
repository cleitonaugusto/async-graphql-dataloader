use async_trait::async_trait;
use sqlx::{Pool, Postgres, FromRow};
use std::collections::HashMap;
use super::super::loader::BatchLoad;

#[derive(FromRow, Clone, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

pub struct UserLoader {
    pool: Pool<Postgres>,
}

impl UserLoader {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BatchLoad for UserLoader {
    type Key = i32;
    type Value = User;
    type Error = String;

    async fn load(&self, keys: &[i32]) -> HashMap<i32, Result<User, String>> {
        if keys.is_empty() {
            return HashMap::new();
        }

        // Cria placeholders para a query IN
        let placeholders: Vec<String> = (0..keys.len())
            .map(|i| format!("${}", i + 1))
            .collect();
        let query = format!(
            "SELECT id, name, email FROM users WHERE id IN ({})",
            placeholders.join(", ")
        );

        // Prepara os parâmetros
        let mut query = sqlx::query_as::<_, User>(&query);
        for key in keys {
            query = query.bind(key);
        }

        match query.fetch_all(&self.pool).await {
            Ok(users) => {
                users.into_iter()
                    .map(|user| (user.id, Ok(user)))
                    .collect()
            }
            Err(e) => {
                // Retorna erro para todas as keys
                keys.iter()
                    .map(|&key| (key, Err(format!("Database error: {}", e))))
                    .collect()
            }
        }
    }
}

// Loader genérico para qualquer tabela
pub struct GenericSQLLoader<T, K> 
where 
    T: for<'r> FromRow<'r> + Send + Unpin + Clone + 'static,
    K: From<i32> + Into<i32> + Clone + Eq + std::hash::Hash + 'static,
{
    pool: Pool<Postgres>,
    table_name: String,
    id_column: String,
    _phantom: std::marker::PhantomData<(T, K)>,
}

impl<T, K> GenericSQLLoader<T, K>
where
    T: for<'r> FromRow<'r> + Send + Unpin + Clone + 'static,
    K: From<i32> + Into<i32> + Clone + Eq + std::hash::Hash + 'static,
{
    pub fn new(pool: Pool<Postgres>, table_name: &str, id_column: &str) -> Self {
        Self {
            pool,
            table_name: table_name.to_string(),
            id_column: id_column.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T, K> BatchLoad for GenericSQLLoader<T, K>
where
    T: for<'r> FromRow<'r> + Send + Unpin + Clone + 'static,
    K: From<i32> + Into<i32> + Clone + Eq + std::hash::Hash + 'static,
{
    type Key = K;
    type Value = T;
    type Error = String;

    async fn load(&self, keys: &[K]) -> HashMap<K, Result<T, String>> {
        if keys.is_empty() {
            return HashMap::new();
        }

        let ids: Vec<i32> = keys.iter().map(|k| (*k).clone().into()).collect();
        let placeholders: Vec<String> = (0..keys.len())
            .map(|i| format!("${}", i + 1))
            .collect();

        let query = format!(
            "SELECT * FROM {} WHERE {} IN ({})",
            self.table_name, self.id_column, placeholders.join(", ")
        );

        let mut query = sqlx::query_as::<_, T>(&query);
        for id in &ids {
            query = query.bind(id);
        }

        match query.fetch_all(&self.pool).await {
            Ok(records) => {
                // Assume que a primeira coluna é o ID
                records.into_iter()
                    .map(|record| {
                        // Aqui você precisaria de um way para extrair o ID do record
                        // Isso depende da sua estrutura de dados específica
                        todo!("Implementar extração de ID baseado na estrutura T")
                    })
                    .collect()
            }
            Err(e) => {
                keys.iter()
                    .map(|key| (key.clone(), Err(format!("Database error: {}", e))))
                    .collect()
            }
        }
    }
}
