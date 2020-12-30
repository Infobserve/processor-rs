mod connection;

use r2d2_postgres::postgres::Transaction;
use anyhow::Result;

pub use connection::{Client, DbConnection};


pub trait Insert {
    fn insert(&mut self, conn: &mut Transaction) -> Result<()>;
}
