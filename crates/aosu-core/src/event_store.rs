//! Persistent, queryable event store (SQLite, local file, no cloud
//! dependency). Builds depth on the one thing that actually works in
//! production today -- real push-notification events, see
//! `docs/event-pipeline.md` -- rather than more transport speculation.
//!
//! Design: the JSONL file `tools/notification_bridge.py --jsonl-out`
//! writes stays the dumb, append-only, human-greppable raw capture.
//! This store is a *derived*, queryable, retained cache built from it
//! -- ingestion is idempotent (same line twice is a no-op), so it's
//! safe to re-ingest the whole file on every open.
//!
//! Schema versioning: see `SCHEMA_VERSION`. A future incompatible
//! change bumps it and adds a migration in `migrate()`; this module
//! never silently reads a newer/older schema as if it matched.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Bump on any incompatible schema change and add a migration step.
pub const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: String,
    pub timestamp_ms: i64,
    pub event_type: String,
    pub channel: String,
    pub camera_name: Option<String>,
    pub confidence: String,
    pub classification_note: Option<String>,
    /// Never discarded -- the exact JSONL line this row came from.
    pub raw_json: String,
}

#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    pub camera_name: Option<String>,
    pub event_type: Option<String>,
    pub since_ms: Option<i64>,
    pub until_ms: Option<i64>,
    /// Case-insensitive substring match against the raw JSON (covers
    /// text search across whatever the raw payload contains, without
    /// needing a fixed set of "searchable" columns).
    pub search_text: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl EventQuery {
    pub fn page(limit: i64, offset: i64) -> Self {
        Self {
            limit,
            offset,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceActivity {
    pub camera_name: String,
    pub event_count: i64,
    pub distinct_event_types: Vec<String>,
    pub last_seen_ms: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventAnalytics {
    pub events_today: i64,
    pub by_camera: Vec<(String, i64)>,
    pub by_type: Vec<(String, i64)>,
    /// Count per UTC day for the last 14 days, oldest first. "Trend"
    /// kept intentionally simple (a count series, not a forecast) --
    /// see North Star note in docs/ROADMAP.md about building depth on
    /// real data, not speculative sophistication.
    pub daily_counts_last_14_days: Vec<(String, i64)>,
}

pub struct EventStore {
    conn: Connection,
}

fn row_id(line: &str) -> String {
    let mut hasher = DefaultHasher::new();
    line.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

impl EventStore {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_meta (version INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS events (
                 id TEXT PRIMARY KEY,
                 timestamp_ms INTEGER NOT NULL,
                 event_type TEXT NOT NULL,
                 channel TEXT NOT NULL,
                 camera_name TEXT,
                 confidence TEXT NOT NULL,
                 classification_note TEXT,
                 raw_json TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp_ms);
             CREATE INDEX IF NOT EXISTS idx_events_camera ON events(camera_name);
             CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);",
        )?;
        let current: Option<i64> = self
            .conn
            .query_row("SELECT version FROM schema_meta LIMIT 1", [], |r| r.get(0))
            .optional()?;
        match current {
            None => {
                self.conn.execute(
                    "INSERT INTO schema_meta (version) VALUES (?1)",
                    params![SCHEMA_VERSION],
                )?;
            }
            Some(v) if v == SCHEMA_VERSION => {}
            Some(v) => {
                // No migrations exist yet (SCHEMA_VERSION has never
                // changed) -- a mismatch here means a future version
                // of this code opened an old/newer file without a
                // migration step having been written. Fail loudly
                // rather than silently misreading rows.
                return Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_SCHEMA),
                    Some(format!(
                        "event store schema version {v} != {SCHEMA_VERSION}, no migration defined"
                    )),
                ));
            }
        }
        Ok(())
    }

    /// Parses and inserts one JSONL line (the format `event_pipeline.py`
    /// writes). Returns `Ok(true)` if newly inserted, `Ok(false)` if
    /// already present (idempotent) or the line was empty/whitespace,
    /// `Err` only for a real database error -- a line that isn't valid
    /// JSON is reported via the `Result<_, ParseError>` wrapper from
    /// `ingest_jsonl_str`, not silently merged into this boolean.
    fn insert_line(&self, line: &str) -> Result<InsertOutcome, IngestError> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(InsertOutcome::Skipped);
        }
        let value: serde_json::Value =
            serde_json::from_str(trimmed).map_err(|e| IngestError::Malformed(e.to_string()))?;
        let event_type = value
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("UnknownVendorEvent")
            .to_string();
        let channel = value
            .get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let camera_name = value
            .get("camera_name")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let confidence = value
            .get("confidence")
            .and_then(|v| v.as_str())
            .unwrap_or("none")
            .to_string();
        let classification_note = value
            .get("classification_note")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let timestamp_ms = value
            .get("timestamp_ms")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let id = row_id(trimmed);
        let changed = self
            .conn
            .execute(
                "INSERT OR IGNORE INTO events
                 (id, timestamp_ms, event_type, channel, camera_name, confidence, classification_note, raw_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    timestamp_ms,
                    event_type,
                    channel,
                    camera_name,
                    confidence,
                    classification_note,
                    trimmed
                ],
            )
            .map_err(IngestError::Database)?;
        Ok(if changed > 0 {
            InsertOutcome::Inserted
        } else {
            InsertOutcome::Duplicate
        })
    }

    /// Ingests every line of a JSONL file. Malformed lines are counted
    /// and reported, never fatal to the rest of the file -- consistent
    /// with "unknown events must remain visible", a malformed *line*
    /// (not a recognized-but-unknown event) is the one case genuinely
    /// unrepresentable, so it's surfaced as a count, not silently eaten.
    pub fn ingest_jsonl_file(&self, path: impl AsRef<Path>) -> std::io::Result<IngestReport> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.ingest_jsonl_str(&content))
    }

    pub fn ingest_jsonl_str(&self, content: &str) -> IngestReport {
        const MAX_SAMPLES: usize = 20;
        let mut report = IngestReport::default();
        for line in content.lines() {
            match self.insert_line(line) {
                Ok(InsertOutcome::Inserted) => report.inserted += 1,
                Ok(InsertOutcome::Duplicate) => report.duplicates += 1,
                Ok(InsertOutcome::Skipped) => {}
                Err(e @ (IngestError::Malformed(_) | IngestError::Database(_))) => {
                    report.malformed += 1;
                    if report.malformed_samples.len() < MAX_SAMPLES {
                        report.malformed_samples.push(e.to_string());
                    }
                }
            }
        }
        report
    }

    pub fn query(&self, q: &EventQuery) -> rusqlite::Result<Vec<StoredEvent>> {
        let mut sql = String::from(
            "SELECT id, timestamp_ms, event_type, channel, camera_name, confidence, classification_note, raw_json FROM events WHERE 1=1",
        );
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(cam) = &q.camera_name {
            sql.push_str(" AND camera_name = ?");
            args.push(Box::new(cam.clone()));
        }
        if let Some(t) = &q.event_type {
            sql.push_str(" AND event_type = ?");
            args.push(Box::new(t.clone()));
        }
        if let Some(since) = q.since_ms {
            sql.push_str(" AND timestamp_ms >= ?");
            args.push(Box::new(since));
        }
        if let Some(until) = q.until_ms {
            sql.push_str(" AND timestamp_ms <= ?");
            args.push(Box::new(until));
        }
        if let Some(search) = &q.search_text {
            sql.push_str(" AND raw_json LIKE ?");
            args.push(Box::new(format!("%{search}%")));
        }
        sql.push_str(" ORDER BY timestamp_ms DESC LIMIT ? OFFSET ?");
        args.push(Box::new(q.limit.max(0)));
        args.push(Box::new(q.offset.max(0)));

        let mut stmt = self.conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(StoredEvent {
                id: row.get(0)?,
                timestamp_ms: row.get(1)?,
                event_type: row.get(2)?,
                channel: row.get(3)?,
                camera_name: row.get(4)?,
                confidence: row.get(5)?,
                classification_note: row.get(6)?,
                raw_json: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    pub fn count_total(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
    }

    pub fn device_activity(&self) -> rusqlite::Result<Vec<DeviceActivity>> {
        let mut stmt = self.conn.prepare(
            "SELECT camera_name, COUNT(*), MAX(timestamp_ms)
             FROM events WHERE camera_name IS NOT NULL
             GROUP BY camera_name ORDER BY MAX(timestamp_ms) DESC",
        )?;
        let rows: Vec<(String, i64, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;

        let mut result = Vec::with_capacity(rows.len());
        for (camera_name, event_count, last_seen_ms) in rows {
            let mut type_stmt = self.conn.prepare(
                "SELECT DISTINCT event_type FROM events WHERE camera_name = ?1 ORDER BY event_type",
            )?;
            let types: Vec<String> = type_stmt
                .query_map(params![camera_name], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            result.push(DeviceActivity {
                camera_name,
                event_count,
                distinct_event_types: types,
                last_seen_ms,
            });
        }
        Ok(result)
    }

    pub fn analytics(&self, now_ms: i64) -> rusqlite::Result<EventAnalytics> {
        let day_ms = 86_400_000i64;
        let today_start = (now_ms / day_ms) * day_ms;

        let events_today: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM events WHERE timestamp_ms >= ?1",
            params![today_start],
            |r| r.get(0),
        )?;

        let mut by_camera_stmt = self.conn.prepare(
            "SELECT camera_name, COUNT(*) FROM events WHERE camera_name IS NOT NULL GROUP BY camera_name ORDER BY COUNT(*) DESC",
        )?;
        let by_camera: Vec<(String, i64)> = by_camera_stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;

        let mut by_type_stmt = self.conn.prepare(
            "SELECT event_type, COUNT(*) FROM events GROUP BY event_type ORDER BY COUNT(*) DESC",
        )?;
        let by_type: Vec<(String, i64)> = by_type_stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;

        let mut daily_counts = Vec::with_capacity(14);
        for day_offset in (0..14).rev() {
            let day_start = today_start - day_offset * day_ms;
            let day_end = day_start + day_ms;
            let count: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM events WHERE timestamp_ms >= ?1 AND timestamp_ms < ?2",
                params![day_start, day_end],
                |r| r.get(0),
            )?;
            let label = format_day_label(day_start);
            daily_counts.push((label, count));
        }

        Ok(EventAnalytics {
            events_today,
            by_camera,
            by_type,
            daily_counts_last_14_days: daily_counts,
        })
    }

    /// Deletes events older than `max_age_days` and/or beyond
    /// `max_rows` (keeping the newest). Both are optional and
    /// user-configurable, applying neither is a no-op, not an error.
    /// Returns the number of rows deleted.
    pub fn apply_retention(
        &self,
        now_ms: i64,
        max_age_days: Option<u32>,
        max_rows: Option<i64>,
    ) -> rusqlite::Result<i64> {
        let mut deleted = 0i64;
        if let Some(days) = max_age_days {
            let cutoff = now_ms - (days as i64) * 86_400_000;
            deleted += self.conn.execute(
                "DELETE FROM events WHERE timestamp_ms < ?1",
                params![cutoff],
            )? as i64;
        }
        if let Some(max) = max_rows {
            let total = self.count_total()?;
            if total > max {
                let excess = total - max;
                deleted += self.conn.execute(
                    "DELETE FROM events WHERE id IN (
                         SELECT id FROM events ORDER BY timestamp_ms ASC LIMIT ?1
                     )",
                    params![excess],
                )? as i64;
            }
        }
        Ok(deleted)
    }

    pub fn export_json(&self, q: &EventQuery) -> rusqlite::Result<String> {
        let events = self.query(q)?;
        Ok(serde_json::to_string_pretty(&events).unwrap_or_default())
    }

    pub fn export_csv(&self, q: &EventQuery) -> rusqlite::Result<String> {
        let events = self.query(q)?;
        let mut out = String::from("id,timestamp_ms,event_type,channel,camera_name,confidence\n");
        for e in events {
            out.push_str(&format!(
                "{},{},{},{},{},{}\n",
                csv_escape(&e.id),
                e.timestamp_ms,
                csv_escape(&e.event_type),
                csv_escape(&e.channel),
                csv_escape(e.camera_name.as_deref().unwrap_or("")),
                csv_escape(&e.confidence),
            ));
        }
        Ok(out)
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn format_day_label(day_start_ms: i64) -> String {
    // Minimal, dependency-free YYYY-MM-DD from a UTC-midnight
    // millisecond timestamp -- avoids pulling in a datetime crate for
    // one label format.
    let days_since_epoch = day_start_ms / 86_400_000;
    let (y, m, d) = civil_from_days(days_since_epoch);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant's days-from-civil algorithm, inverted (days_from_civil
/// -> civil_from_days), public domain. Avoids a chrono dependency.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

enum InsertOutcome {
    Inserted,
    Duplicate,
    Skipped,
}

enum IngestError {
    Malformed(String),
    Database(rusqlite::Error),
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IngestError::Malformed(reason) => write!(f, "malformed JSONL line: {reason}"),
            IngestError::Database(e) => write!(f, "database error: {e}"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestReport {
    pub inserted: u64,
    pub duplicates: u64,
    pub malformed: u64,
    /// Up to 20 reasons, for diagnostics visibility -- a malformed line
    /// is reported, not silently swallowed into just a number.
    pub malformed_samples: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_line(camera: &str, event_type: &str, ts: i64) -> String {
        format!(
            r#"{{"event_type": "{event_type}", "camera_name": "{camera}", "channel": "com.glazero.android.event.v2", "timestamp_ms": {ts}, "confidence": "low", "classification_note": "n"}}"#
        )
    }

    #[test]
    fn opens_in_memory_and_starts_empty() {
        let store = EventStore::open_in_memory().unwrap();
        assert_eq!(store.count_total().unwrap(), 0);
    }

    #[test]
    fn ingests_the_real_anonymized_fixture() {
        let store = EventStore::open_in_memory().unwrap();
        let fixture = include_str!("../tests/fixtures/real_capture_anonymized.jsonl");
        let report = store.ingest_jsonl_str(fixture);
        assert_eq!(report.inserted, 1);
        assert_eq!(report.malformed, 0);
        let events = store.query(&EventQuery::page(10, 0)).unwrap();
        assert_eq!(events[0].camera_name.as_deref(), Some("Kamera 1"));
        assert_eq!(events[0].event_type, "MotionDetected");
    }

    #[test]
    fn re_ingesting_the_same_content_is_idempotent() {
        let store = EventStore::open_in_memory().unwrap();
        let content = sample_line("A", "MotionDetected", 1000);
        let r1 = store.ingest_jsonl_str(&content);
        let r2 = store.ingest_jsonl_str(&content);
        assert_eq!(r1.inserted, 1);
        assert_eq!(r2.inserted, 0);
        assert_eq!(r2.duplicates, 1);
        assert_eq!(store.count_total().unwrap(), 1);
    }

    #[test]
    fn malformed_lines_are_counted_not_fatal() {
        let store = EventStore::open_in_memory().unwrap();
        let content = format!("not json\n{}\n", sample_line("A", "MotionDetected", 1000));
        let report = store.ingest_jsonl_str(&content);
        assert_eq!(report.malformed, 1);
        assert_eq!(report.inserted, 1);
    }

    #[test]
    fn malformed_line_reason_is_visible_in_the_report_not_just_a_count() {
        let store = EventStore::open_in_memory().unwrap();
        let report = store.ingest_jsonl_str("not json at all");
        assert_eq!(report.malformed, 1);
        assert_eq!(report.malformed_samples.len(), 1);
        assert!(report.malformed_samples[0].contains("malformed"));
    }

    #[test]
    fn unknown_event_type_remains_queryable_not_dropped() {
        let store = EventStore::open_in_memory().unwrap();
        store.ingest_jsonl_str(&sample_line("A", "UnknownVendorEvent", 1000));
        let events = store.query(&EventQuery::page(10, 0)).unwrap();
        assert_eq!(events[0].event_type, "UnknownVendorEvent");
    }

    #[test]
    fn query_filters_by_camera() {
        let store = EventStore::open_in_memory().unwrap();
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 1000));
        store.ingest_jsonl_str(&sample_line("B", "MotionDetected", 2000));
        let q = EventQuery {
            camera_name: Some("A".to_string()),
            limit: 10,
            ..Default::default()
        };
        let events = store.query(&q).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].camera_name.as_deref(), Some("A"));
    }

    #[test]
    fn query_filters_by_time_range() {
        let store = EventStore::open_in_memory().unwrap();
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 1000));
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 5000));
        let q = EventQuery {
            since_ms: Some(4000),
            limit: 10,
            ..Default::default()
        };
        let events = store.query(&q).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].timestamp_ms, 5000);
    }

    #[test]
    fn query_supports_pagination() {
        let store = EventStore::open_in_memory().unwrap();
        for i in 0..5 {
            store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 1000 + i));
        }
        let page1 = store.query(&EventQuery::page(2, 0)).unwrap();
        let page2 = store.query(&EventQuery::page(2, 2)).unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 2);
        assert_ne!(page1[0].id, page2[0].id);
    }

    #[test]
    fn query_search_text_matches_raw_payload() {
        let store = EventStore::open_in_memory().unwrap();
        store.ingest_jsonl_str(&sample_line("Garten", "MotionDetected", 1000));
        store.ingest_jsonl_str(&sample_line("Terasse", "MotionDetected", 2000));
        let q = EventQuery {
            search_text: Some("Garten".to_string()),
            limit: 10,
            ..Default::default()
        };
        let events = store.query(&q).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn device_activity_aggregates_per_camera() {
        let store = EventStore::open_in_memory().unwrap();
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 1000));
        store.ingest_jsonl_str(&sample_line("A", "TamperDetected", 2000));
        store.ingest_jsonl_str(&sample_line("B", "MotionDetected", 1500));
        let activity = store.device_activity().unwrap();
        let a = activity.iter().find(|d| d.camera_name == "A").unwrap();
        assert_eq!(a.event_count, 2);
        assert_eq!(a.last_seen_ms, 2000);
        assert_eq!(
            a.distinct_event_types,
            vec!["MotionDetected", "TamperDetected"]
        );
    }

    #[test]
    fn retention_by_age_deletes_old_rows() {
        let store = EventStore::open_in_memory().unwrap();
        let day = 86_400_000i64;
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 0));
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 10 * day));
        let deleted = store.apply_retention(10 * day, Some(5), None).unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(store.count_total().unwrap(), 1);
    }

    #[test]
    fn retention_by_row_count_keeps_newest() {
        let store = EventStore::open_in_memory().unwrap();
        for i in 0..5 {
            store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 1000 + i));
        }
        let deleted = store.apply_retention(999_999_999, None, Some(2)).unwrap();
        assert_eq!(deleted, 3);
        let remaining = store.query(&EventQuery::page(10, 0)).unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.iter().all(|e| e.timestamp_ms >= 1002));
    }

    #[test]
    fn retention_with_no_limits_is_a_no_op() {
        let store = EventStore::open_in_memory().unwrap();
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 1000));
        let deleted = store.apply_retention(999_999, None, None).unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(store.count_total().unwrap(), 1);
    }

    #[test]
    fn analytics_counts_events_today_and_by_camera_and_type() {
        let store = EventStore::open_in_memory().unwrap();
        let day = 86_400_000i64;
        let now = 20 * day + 1000;
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 20 * day + 500));
        store.ingest_jsonl_str(&sample_line("A", "TamperDetected", 19 * day + 500));
        store.ingest_jsonl_str(&sample_line("B", "MotionDetected", 20 * day + 600));
        let analytics = store.analytics(now).unwrap();
        assert_eq!(analytics.events_today, 2);
        assert_eq!(analytics.by_camera.len(), 2);
        assert_eq!(analytics.by_type.len(), 2);
        assert_eq!(analytics.daily_counts_last_14_days.len(), 14);
        assert_eq!(analytics.daily_counts_last_14_days.last().unwrap().1, 2);
    }

    #[test]
    fn export_json_round_trips_preserving_raw_payload() {
        let store = EventStore::open_in_memory().unwrap();
        store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 1000));
        let json = store.export_json(&EventQuery::page(10, 0)).unwrap();
        let parsed: Vec<StoredEvent> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].raw_json.contains("MotionDetected"));
    }

    #[test]
    fn export_csv_escapes_commas_in_camera_names() {
        let store = EventStore::open_in_memory().unwrap();
        store.ingest_jsonl_str(&sample_line("Garten, Süd", "MotionDetected", 1000));
        let csv = store.export_csv(&EventQuery::page(10, 0)).unwrap();
        assert!(csv.contains("\"Garten, Süd\""));
    }

    #[test]
    fn opening_the_same_file_twice_reuses_existing_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.sqlite");
        {
            let store = EventStore::open(&path).unwrap();
            store.ingest_jsonl_str(&sample_line("A", "MotionDetected", 1000));
        }
        let store2 = EventStore::open(&path).unwrap();
        assert_eq!(store2.count_total().unwrap(), 1);
    }
}
