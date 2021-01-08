#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Local, TimeZone};
use r2d2_postgres::postgres::{Row, Transaction};
use serde_json::Value;

use crate::database::Insert;
use crate::errors::ProcessingError;

const DATETIME_FMT: &str = "%Y/%m/%d-%H:%M:%S";

#[derive(Debug)]
pub struct Event {
    /// A unique identifier for this event. Could be of different format based on the source
    id: Option<i32>,
    /// The url from which this file was retrieved
    url: String,
    /// The size of the file (in bytes)
    size: i64,
    /// The source in which the paste was found
    source: String,
    /// The entire content of the file as retrieved from the source
    raw_content: String,
    /// The name of the file as retrieved from the source
    filename: String,
    /// The username of the creator
    creator: String,
    /// Time at which the paste was created
    created_at: DateTime<Local>,
    /// Time at which the paste was discovered
    discovered_at: DateTime<Local>
}

impl Insert for Event {
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
                &self.size,
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

    // raw_content: String,
    // filename: String,
    // creator: String,
    // created_at: DateTime<Local>,
    // discovered_at: DateTime<Local>
        let url = match json["url"].as_str() {
            Some(u) => u,
            None => return Err(ProcessingError::NoValueError("url".to_string()).into())
        };
        let size = match json["size"].as_i64() {
            Some(s) => s,
            None => return Err(ProcessingError::NoValueError("size".to_string()).into())
        };
        let source = match json["source"].as_str() {
            Some(s) => s,
            None => return Err(ProcessingError::NoValueError("source".to_string()).into())
        };
        let raw_content = match json["raw_content"].as_str() {
            Some(r) => r,
            None => return Err(ProcessingError::NoValueError("raw_content".to_string()).into())
        };
        let filename = match json["filename"].as_str() {
            Some(f) => f,
            None => return Err(ProcessingError::NoValueError("filename".to_string()).into())
        };
        let creator = match json["creator"].as_str() {
            Some(c) => c,
            None => return Err(ProcessingError::NoValueError("creator".to_string()).into())
        };
        let created_at: DateTime<Local> = match json["created_at"].as_str() {
            Some(c) => {
                match Local.datetime_from_str( c, DATETIME_FMT) {
                    Ok(d) => d,
                    Err(e) => return Err(e.into())
                }
            },
            None => return Err(ProcessingError::NoValueError("created_at".to_string()).into())
        };
        let discovered_at: DateTime<Local> = match json["discovered_at"].as_str() {
            Some(d) => {
                match Local.datetime_from_str(d, DATETIME_FMT) {
                    Ok(d) => d,
                    Err(e) => return Err(e.into())
                }
            },
            None => return Err(ProcessingError::NoValueError("discovered_at".to_string()).into())
        };

        Ok(Self::new(url, size, source, raw_content, filename, creator, created_at, discovered_at))
    }

    pub fn new(
        url: &str,
        size: i64,
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
            row.get("size"),
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

    pub fn size(&self) -> i64 {
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
        size: i64,
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
}
