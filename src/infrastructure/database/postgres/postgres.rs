use chrono::Utc;
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions};

use crate::infrastructure::database::{DatabaseError, DatabaseOptions};

#[non_exhaustive]
pub struct PostgresDatabase {
    pool: PgPool,
    options: DatabaseOptions,
    test_db_to_drop: Option<String>,
}

impl PostgresDatabase {
    pub async fn connect(options: DatabaseOptions) -> Result<Self, DatabaseError> {
        // Get postgres configuration.
        let connection_url = options.postgres.connection_url();
        let max_connections = options.postgres.max_connections();

        // Connect to the database and get a connection pool.
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&connection_url)
            .await?;

        tracing::info!("Connected to PostgreSQL database.");

        Ok(Self {
            pool,
            options,
            test_db_to_drop: None,
        })
    }

    pub async fn connect_test(options: DatabaseOptions) -> Result<Self, DatabaseError> {
        // Generate a temporary name for the test database.
        let nanos_since_epoch = Utc::now().timestamp_nanos_opt().unwrap();
        let test_db_name = format!("tmp_{:x}", nanos_since_epoch);

        // Create a temporary database.
        {
            let mut options = options.clone();
            options.postgres.set_max_connections(1);
            let db = Self::connect(options).await?;
            let pool = db.pool();
            let query = format!("CREATE DATABASE {}", postgres_identifier(&test_db_name));
            sqlx::query(AssertSqlSafe(query)).execute(pool).await?;
        }

        // Connect to the temporary database.
        let mut test_options = options.clone();
        test_options.postgres.set_db(&test_db_name);
        let mut test_db = Self::connect(test_options).await?;

        // Prepare for the drop on close.
        test_db.test_db_to_drop = Some(test_db_name.to_owned());
        test_db.options = options;

        Ok(test_db)
    }

    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn drop(&self) -> Result<(), DatabaseError> {
        if let Some(test_db_to_drop) = self.test_db_to_drop.as_ref() {
            // Close connections.
            self.pool.close().await;

            // Drop the temporary database.
            let db = Self::connect(self.options.clone()).await?;
            let pool = db.pool();
            let query = format!(
                "DROP DATABASE IF EXISTS {} WITH (FORCE)",
                postgres_identifier(test_db_to_drop)
            );
            sqlx::query(AssertSqlSafe(query)).execute(pool).await?;
        }
        Ok(())
    }
}

fn postgres_identifier(identifier: &str) -> String {
    format!(r#""{}""#, identifier.replace('"', "\"\""))
}
