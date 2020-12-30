#![allow(dead_code)]

use r2d2_postgres::postgres::{Row, Transaction};
use anyhow::Result;
use crate::database::{Client, Insert};
use crate::entities::Event;

#[derive(Debug)]
pub struct RuleMatch {
    id: Option<i32>,
    event_id: i32,
    rule_matched: String,
    tags_matched: Vec<String>
}

impl Insert for RuleMatch {
    fn insert(&mut self, conn: &mut Transaction) -> Result<()> {
        let stmt = "
        INSERT INTO rule_matches
        (
            event_id,
            rule_matched,
            tags_matched
        )
        VALUES
        (
            $1, $2, $3
        )
        RETURNING id
        ";

        let row = conn.query_one(stmt, &[&self.event_id, &self.rule_matched, &self.tags_matched])?;
        self.id = row.get(0);

        Ok(())
    }
}

impl RuleMatch {
    pub fn new(event_id: i32, rule_matched: String, tags_matched: Vec<String>) -> Self {
        Self::create(None, event_id, rule_matched, tags_matched)
    }

    pub fn from_row(row: &Row) -> Self {
        Self::create(
            Some(row.get("id")),
            row.get("event_id"),
            row.get("rule_matched"),
            row.get("tags_matched")
        )
    }

    pub fn event(&self, conn: &mut Client) -> Result<Event> {
        let row = conn.query_one("SELECT * FROM events WHERE id = $1", &[&self.event_id])?;

        Ok(Event::from_row(row))
    }

    pub fn id(&self) -> Option<i32> {
        self.id
    }

    pub fn event_id(&self) -> i32 {
        self.event_id
    }

    pub fn rule_matched(&self) -> &str {
        &self.rule_matched
    }

    pub fn tags_matched(&self) -> &[String] {
        &self.tags_matched
    }

    fn create(id: Option<i32>, event_id: i32, rule_matched: String, tags_matched: Vec<String>) -> Self {
        Self { id, event_id, rule_matched, tags_matched }
    }
}
