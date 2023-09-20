//! Handles the loading of processed events into the database
//! Splits the events appropriately (Events -> RuleMatches -> AsciiMatches)
//! and inserts them into the DB
extern crate r2d2;

use std::{fs, error, thread, sync};
use log::{info, error};

use crossbeam_channel::Receiver;
use anyhow::Result;

use crate::entities::{RuleMatch, ProcessedEvent, AsciiMatch};
use crate::database::{DbConnection, Insert};
use crate::utils;

/// Given the consuming end of a crossbeam channel, continuously consumes
/// ProcessedEvent objects and stores them in the db.
/// This work happens in N threads
/// Returns a vector of the spawned thread handles
///
/// # Example
/// 
/// ```
/// use crate::database::{DbConnection, DbLoader};
/// use crate::entities::ProcessedEvent;
///
/// let conn = DbConnection::connect("foo", "bar", "baz", "localhost", 12345)
/// let loader = DbLoader::with_connection(conn);
/// let (receiver, sender) = crossbeam_channel::unbounded();
///
/// let handles = start_loaders(&receiver, loader, 4);
/// 
/// assert_eq!(handles.len(), 4);
/// // let pevent = ProcessedEvent(...)
/// // sender.send(pevent);
/// // The sender *has* to be dropped for the threads to cleanly return
/// drop(sender);
/// 
/// for handle in handles {
///     handle.join.unwrap();
/// }
/// ```
pub fn start_loaders(
    load_recvr: &Receiver<ProcessedEvent>,
    db_loader: DbLoader,
    num_loaders: i32
) -> Vec<thread::JoinHandle<()>> {
    if num_loaders == 0 {
        let msg = "Refusing to continue with 0 loaders -- Process would hang";
        error!("{}", msg);
        panic!("{}", msg);
    }

    let mut l_handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity(num_loaders as usize);
    let db_loader_arc = sync::Arc::new(db_loader);

    info!("Spawning {} DB loaders", num_loaders);
    for _ in 0..num_loaders {
        let rx = crossbeam_channel::Receiver::clone(load_recvr);
        let db_loader = sync::Arc::clone(&db_loader_arc);

        l_handles.push(
            thread::spawn(move || {
                for proc_event in rx {
                    db_loader.persist_processed_event(proc_event);
                }
            })
        );
    }

    l_handles
}

pub struct DbLoader {
    conn: DbConnection
}

impl DbLoader {
    pub fn with_connection(conn: DbConnection) -> Self {
        Self { conn }
    }

    /// Reads and loads the infobserve schema from the "infobserve-schema.sql"
    /// file
    pub fn create_schema(&self) -> Result<(), Box<dyn error::Error>> {
        let mut client = self.conn.get()?;

        info!("Creating initial infobserve schema");
        let contents = match fs::read_to_string("infobserve-schema.sql") {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load infobserve schema file: {}", e);
                return Err(e.into());
            }
        };

        if let Err(e) = client.simple_query(&contents) {
            error!("Failed to create infobserve schema: {}", e);
            return Err(Box::new(e));
        }

        Ok(())
    }

    pub fn persist_processed_event(&self, proc_event: ProcessedEvent) {
        // TODO: All these should be in a transaction
        // I should pick up here and check how transactions in
        // postgres-rs work (https://docs.rs/postgres/0.15.2/postgres/transaction/struct.Transaction.html)
        info!("Persisting {:?}", proc_event);

        let mut client = match self.conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get connection: {}", e);
                return;
            }
        };

        let mut trans = match client.transaction() {
            Ok(t) => t,
            Err(e) => {
                error!("Could not initiate transaction to db: {}", e);
                return;
            }
        };

        let ProcessedEvent(mut event, matches) = proc_event;

        if let Err(e) = event.insert(&mut trans) {
            error!("Failed to insert event: {}", e);
            return;
        }

        let event_id = match event.id() {
            Some(id) => id,
            None => {
                error!("Inserted event has empty ID? {:?}", event);
                return;
            }
        };

        for flat_match in matches {
            let mut rule_match = RuleMatch::new(
                // TODO: unwrap ID before usage and handle errors
                event_id, flat_match.rule_name().to_owned(),
                flat_match.tags().into()
            );

            if let Err(e) = rule_match.insert(&mut trans) {
                error!("Failed to insert rule match: {}", e);
                return;
            }

            let match_id = match rule_match.id() {
                Some(id) => id,
                None => {
                    error!("Inserted rule match has empty ID? {:?}", rule_match);
                    return;
                }
            };

            for data in flat_match.data() {
                let mut ascii_match = AsciiMatch::new(match_id, data.to_owned());

                if let Err(e) = ascii_match.insert(&mut trans) {
                    error!("Failed to insert ascii match: {}", e);
                    return;
                }
            }
        }

        if let Err(e) = trans.commit() {
            error!("Unable to commit transaction: {}", e);
        }
    }
}
