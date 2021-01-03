//! Handles the initial connection to Postgres as well as handing out
//! connections where needed.
//! It is a *very* thin wrapper around the r2d2 connection pool, using the
//! postgres driver
//! 
//! # Example
//! 
//! ```
//! use crate::database::DbConnection;
//! 
//! fn insert_stuff(conn: &DbConnection) {
//!     let mut client = conn.get().unwrap();
//!     client.execute("INSERT INTO foo (value) VALUES ($1)", &[&"bar"]);
//!     // When `client` goes out of scope, the connection is returned to the pool
//! }
//! 
//! fn select_stuff(conn: &DbConnection) {
//!     let mut client = conn.get().unwrap();
//!     client.query("SELECT * FROM foo");
//!     // When `client` goes out of scope, the connection is returned to the pool
//! }
//! 
//! let conn = DbConnection::connect("user", "password", "database", "localhost", 5432).unwrap();
//! insert_stuff(&conn);
//! select_stuff(&conn);
//! 
//! // When `conn` goes out of scope, all connections are closed
//! ```
extern crate r2d2;

use log::info;

use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use r2d2::{Pool, PooledConnection};
use anyhow::Result;

pub type Client = PooledConnection<NoTlsConnection>;
type NoTlsConnection = PostgresConnectionManager<NoTls>;
type PostgresPool = Pool<NoTlsConnection>;

pub struct DbConnection {
    pool: PostgresPool,
}

impl DbConnection {
    pub fn connect(
        user: &str,
        passwd: &str,
        database: &str,
        host: &str,
        port: u16
    ) -> Result<Self> {
        info!("Connecting to postgres: {}@{}:{}#{}", user, host, port, database);
        let manager = PostgresConnectionManager::new(
            format!("host={} user={} password={} dbname={} port={}", host, user, passwd, database, port).parse()?,
            NoTls
        );

        let pool = r2d2::Pool::new(manager)?;

        Ok(Self { pool })
    }

    pub fn get(&self) -> Result<Client> {
        self.pool.get().map_err(anyhow::Error::new)
    }
}