/*
 * This is the PostgreSQL schema that Infobserve uses to store processed events
 */

CREATE TABLE IF NOT EXISTS events (
  id SERIAL PRIMARY KEY,
  source TEXT, -- The service in which the event was found
  url TEXT, -- The url of the paste
  size BIGINT, -- The size of the paste (in bytes)
  raw_content TEXT, -- The raw text of the event
  filename TEXT, -- The name of the file in which the event was found
  creator TEXT, -- The name of the user that created the post that contained the event
  created_at TIMESTAMPTZ, -- The time and date the event was created
  discovered_at TIMESTAMPTZ -- The time and date the event was discovered
);
CREATE TABLE IF NOT EXISTS rule_matches (
  id SERIAL PRIMARY KEY,
  event_id INTEGER REFERENCES events(id), -- A reference to the event in which the rule matched
  rule_matched TEXT, -- The name of the yara rule that matched
  tags_matched TEXT [] -- The tags of the rule that matched
);
CREATE TABLE IF NOT EXISTS ascii_matches (
  id SERIAL PRIMARY KEY,
  match_id INTEGER REFERENCES rule_matches(id),
  matched_string TEXT -- The matched ASCII string
);
CREATE TABLE IF NOT EXISTS index_cache (
  id SERIAL PRIMARY KEY,
  source TEXT,
  source_id TEXT, -- The Reason is each kind of source could have different definition of a unique id format.
  cached_time TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION expire_cached_rows() RETURNS trigger
  LANGUAGE plpgsql
  AS $$
BEGIN
  DELETE FROM index_cache WHERE cached_time < NOW() - INTERVAL '2 hours';
  RETURN NULL;
END;
$$;


DROP TRIGGER IF EXISTS trigger_expire_cached_rows
  ON PUBLIC.INDEX_CACHE;
CREATE TRIGGER trigger_expire_cached_rows
  AFTER INSERT ON index_cache
  EXECUTE PROCEDURE expire_cached_rows();
