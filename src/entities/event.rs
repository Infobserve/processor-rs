#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Local};
use r2d2_postgres::postgres::{Row, Transaction};
use crate::database::Insert;
use crate::entities::FlatMatch;
use serde_json::Value;

use crate::errors::DeserializationError;

const DATETIME_FMT: &str = "%Y/%m/%d-%H:%M:%S";

/// Responsible for the deserialization as well as DB insertion of
/// events. Contains the following fields:
/// 
/// 
/// id - A unique identifier for this event. Could be of different format based on the source
/// url - The url from which this file was retrieved
/// size - The size of the file (in bytes)
/// source - The source in which the paste was found
/// raw_content - The entire content of the file as retrieved from the source
/// filename - The name of the file as retrieved from the source
/// creator - The username of the creator
/// created_at - Time at which the paste was created
/// discovered_at - Time at which the paste was scraped
#[derive(Debug)]
pub struct Event {
    id: Option<i32>,
    url: String,
    size: usize,
    source: String,
    raw_content: String,
    filename: String,
    creator: String,
    created_at: DateTime<Local>,
    discovered_at: DateTime<Local>
}

#[derive(Debug)]
pub struct ProcessedEvent(pub Event, pub Vec<FlatMatch>);

impl Insert for Event {
    /// Insert the event into the DB
    /// 
    /// # Arguments
    /// 
    /// * conn - A currently open (uncommitted) DB transaction
    /// 
    /// # Returns
    /// 
    /// An empty Result
    fn insert(&mut self, conn: &mut Transaction) -> Result<()> {
        let stmt = "
        INSERT INTO events
        (
            source,
            url,
            size,
            raw_content,
            filename,
            creator,
            created_at,
            discovered_at
        )
        VALUES
        (
            $1, $2, $3, $4, $5, $6, $7, $8
        )
        RETURNING id
        ";

        let row = conn.query_one(
            stmt,
            &[
                &self.source,
                &self.url,
                &(self.size as i64),
                &self.raw_content,
                &self.filename,
                &self.creator,
                &self.created_at,
                &self.discovered_at
            ]
        )?;
        self.id = row.get(0);

        Ok(())
    }
}

impl Event {
    pub fn from_json_str(json_str: &str) -> Result<Self> {
        let json: Value = serde_json::from_str(json_str)?;

        let url = Self::get_str(&json, "url")?;
        let size = Self::get_i64(&json, "size")? as usize;
        let source = Self::get_str(&json, "source")?;
        let raw_content = Self::get_str(&json, "raw_content")?;
        let filename = Self::get_str(&json, "filename")?;
        let creator = Self::get_str(&json, "creator")?;
        let created_at: DateTime<Local> = 
            match Self::get_str(&json, "created_at") {
                Ok(c) => DateTime::parse_from_str(&c, DATETIME_FMT)?.into(),
                Err(e) => return Err(e)
                
            };
        let discovered_at: DateTime<Local> =
            match Self::get_str(&json, "discovered_at") {
                Ok(c) => DateTime::parse_from_str(&c, DATETIME_FMT)?.into(),
                Err(e) => return Err(e)
            };

        Ok(Self::new(&url, size, &source, &raw_content, &filename, &creator, created_at, discovered_at))
    }

    pub fn new(
        url: &str,
        size: usize,
        source: &str,
        raw_content: &str,
        filename: &str,
        creator: &str,
        created_at: DateTime<Local>,
        discovered_at: DateTime<Local>
    ) -> Self {
        Self::create( None, url, size, source, raw_content, filename, creator, created_at, discovered_at)
    }

    pub fn from_row(row: Row) -> Self {
        Self::create(
            Some(row.get("id")),
            row.get("url"),
            row.get::<&str, i64>("size") as usize,
            row.get("source"),
            row.get("raw_content"),
            row.get("filename"),
            row.get("creator"),
            row.get("created_at"),
            row.get("discovered_at")
        )
    }

    pub fn id(&self) -> Option<i32> {
        self.id
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn raw_content(&self) -> &str {
        &self.raw_content
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn creator(&self) -> &str {
        &self.creator
    }

    pub fn created_at(&self) -> &DateTime<Local> {
        &self.created_at
    }

    pub fn discovered_at(&self) -> &DateTime<Local> {
        &self.discovered_at
    }

    #[allow(clippy::too_many_arguments)]
    fn create(
        id: Option<i32>,
        url: &str,
        size: usize,
        source: &str,
        raw_content: &str,
        filename: &str,
        creator: &str,
        created_at: DateTime<Local>,
        discovered_at: DateTime<Local>
    ) -> Self {
        Self {
            id,
            url: url.to_owned(),
            size,
            source: source.to_owned(),
            raw_content: raw_content.to_owned(),
            filename: filename.to_owned(),
            creator: creator.to_owned(),
            created_at,
            discovered_at
        }
    }

    fn get_str(json: &Value, field_name: &str) -> Result<String> {
        match json[field_name].as_str() {
            Some(u) => Ok(u.to_owned()),
            None => return Err(DeserializationError::NoValueError(field_name.to_string()).into())
        }
    }

    fn get_i64(json: &Value, field_name: &str) -> Result<i64> {
        match json[field_name].as_i64() {
            Some(u) => Ok(u.to_owned()),
            None => return Err(DeserializationError::NoValueError(field_name.to_string()).into())
        }

    }
}
