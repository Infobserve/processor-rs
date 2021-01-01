#![allow(dead_code)]

use r2d2_postgres::postgres::{Row, Transaction};
use anyhow::Result;
use crate::database::{Client, Insert};
use crate::entities::RuleMatch;

#[derive(Debug)]
pub struct AsciiMatch {
    id: Option<i32>,
    rule_match_id: i32,
    matched_string: String
}

impl Insert for AsciiMatch {
    fn insert(&mut self, conn: &mut Transaction) -> Result<()> {
        let stmt = "
        INSERT INTO ascii_matches
        (
            match_id,
            matched_string
        )
        VALUES
        (
            $1, $2
        )
        RETURNING id
        ";

        let row = conn.query_one(stmt, &[&self.rule_match_id, &self.matched_string])?;
        self.id = row.get(0);

        Ok(())
    }
}

impl AsciiMatch {
    pub fn new(rule_match_id: i32, matched_string: String) -> Self {
        Self::create(None, rule_match_id, matched_string)
    }

    pub fn from_row(row: &Row) -> Self {
        Self::create(
            row.get("id"),
            row.get("rule_match_id"),
            row.get("matched_string")
        )
    }

    pub fn with_id(id: i32, rule_match_id: i32, matched_string: String) -> Self {
        Self::create(Some(id), rule_match_id, matched_string)
    }

    pub fn id(&self) -> Option<i32> {
        self.id
    }

    pub fn rule_match_id(&self) -> i32 {
        self.rule_match_id
    }

    pub fn rule_match(&self, conn: &mut Client) -> Result<RuleMatch> {
        let row = conn.query_one("SELECT * FROM rule_matches WHERE id = $1", &[&self.rule_match_id])?;

        Ok(RuleMatch::from_row(&row))
    }

    pub fn matched_string(&self) -> &str {
        &self.matched_string
    }

    fn create(id: Option<i32>, rule_match_id: i32, matched_string: String) -> Self {
        Self { id, rule_match_id, matched_string }
    }
}
