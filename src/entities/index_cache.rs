#![allow(dead_code)]

use std::time;
use r2d2_postgres::postgres::Transaction;
use anyhow::Result;
use crate::database::Insert;

pub struct IndexCache {
    id: i32,
    source: String,
    source_id: String,
    cached_at: time::SystemTime
}

impl Insert for IndexCache {
    fn insert(&mut self, conn: &mut Transaction) -> Result<()> {
        let stmt = "
        INSERT INTO index_cache
        (
            source,
            source_id,
            cached_at
        )
        VALUES
        (
            $1, $2, $3
        )
        RETURNING id
        ";

        let rows = conn.query(stmt, &[&self.source, &self.source_id, &self.cached_at])?;
        self.id = rows[0].get(0);

        Ok(())
    }
}

impl IndexCache {
    pub fn new(id: i32, source: String, source_id: String) -> Self {
        let cached_at = time::SystemTime::now();

        Self { id, source, source_id, cached_at }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn cached_at(&self) -> &time::SystemTime {
        &self.cached_at
    }
}
