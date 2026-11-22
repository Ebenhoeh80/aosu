"""Persistent, queryable event store -- Python side of the two-language
Event Platform (see docs/event-pipeline.md). Mirrors
crates/aosu-core/src/event_store.rs: same schema, same semantics,
verified against the same real captured fixture. Uses only the
standard library (`sqlite3`) -- no new dependency for Home Assistant.

Each side (this one, used by Home Assistant/notification_bridge.py; the
Rust one, used by the desktop app) maintains its *own* SQLite file
derived from the same JSONL source, rather than both writing to one
shared file concurrently -- avoids a fragile multi-process-writer
requirement while still sharing the same source of truth and schema.
"""
from __future__ import annotations

import csv
import io
import json
import sqlite3
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

SCHEMA_VERSION = 1

_SCHEMA_SQL = """
CREATE TABLE IF NOT EXISTS schema_meta (version INTEGER NOT NULL);
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
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
"""


class SchemaVersionError(RuntimeError):
    pass


@dataclass
class StoredEvent:
    id: str
    timestamp_ms: int
    event_type: str
    channel: str
    camera_name: Optional[str]
    confidence: str
    classification_note: Optional[str]
    raw_json: str


@dataclass
class EventQuery:
    camera_name: Optional[str] = None
    event_type: Optional[str] = None
    since_ms: Optional[int] = None
    until_ms: Optional[int] = None
    search_text: Optional[str] = None
    limit: int = 50
    offset: int = 0


@dataclass
class DeviceActivity:
    camera_name: str
    event_count: int
    distinct_event_types: list[str]
    last_seen_ms: int


@dataclass
class EventAnalytics:
    events_today: int
    by_camera: list[tuple[str, int]] = field(default_factory=list)
    by_type: list[tuple[str, int]] = field(default_factory=list)
    daily_counts_last_14_days: list[tuple[str, int]] = field(default_factory=list)


@dataclass
class IngestReport:
    inserted: int = 0
    duplicates: int = 0
    malformed: int = 0
    malformed_samples: list[str] = field(default_factory=list)


def _row_id(line: str) -> str:
    # Same approach as the Rust side conceptually (deterministic id from
    # exact line content) -- not required to produce identical ids to
    # Rust's hasher (each side maintains its own derived store), only
    # to be internally idempotent.
    import hashlib

    return hashlib.sha256(line.encode("utf-8")).hexdigest()[:16]


class EventStore:
    def __init__(self, path: str | Path = ":memory:"):
        self._conn = sqlite3.connect(str(path))
        self._conn.execute("PRAGMA busy_timeout = 2000")
        self._init_schema()

    @classmethod
    def open_in_memory(cls) -> "EventStore":
        return cls(":memory:")

    def _init_schema(self) -> None:
        self._conn.executescript(_SCHEMA_SQL)
        cur = self._conn.execute("SELECT version FROM schema_meta LIMIT 1")
        row = cur.fetchone()
        if row is None:
            self._conn.execute(
                "INSERT INTO schema_meta (version) VALUES (?)", (SCHEMA_VERSION,)
            )
            self._conn.commit()
        elif row[0] != SCHEMA_VERSION:
            raise SchemaVersionError(
                f"event store schema version {row[0]} != {SCHEMA_VERSION}, "
                "no migration defined"
            )

    def close(self) -> None:
        self._conn.close()

    def count_total(self) -> int:
        return self._conn.execute("SELECT COUNT(*) FROM events").fetchone()[0]

    def _insert_line(self, line: str) -> Optional[bool]:
        """Returns True if inserted, False if duplicate, None if the
        line was blank. Raises ValueError for malformed JSON -- caller
        (ingest_jsonl_str) decides how to report that, consistent with
        "never silently swallow, always report" from docs/event-pipeline.md.
        """
        trimmed = line.strip()
        if not trimmed:
            return None
        data = json.loads(trimmed)  # raises ValueError (json.JSONDecodeError) if malformed
        event_id = _row_id(trimmed)
        cur = self._conn.execute(
            "INSERT OR IGNORE INTO events "
            "(id, timestamp_ms, event_type, channel, camera_name, confidence, classification_note, raw_json) "
            "VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                event_id,
                data.get("timestamp_ms", 0),
                data.get("event_type", "UnknownVendorEvent"),
                data.get("channel", ""),
                data.get("camera_name"),
                data.get("confidence", "none"),
                data.get("classification_note"),
                trimmed,
            ),
        )
        self._conn.commit()
        return cur.rowcount > 0

    def ingest_jsonl_str(self, content: str) -> IngestReport:
        report = IngestReport()
        for line in content.splitlines():
            try:
                outcome = self._insert_line(line)
            except (json.JSONDecodeError, sqlite3.Error) as e:
                report.malformed += 1
                if len(report.malformed_samples) < 20:
                    report.malformed_samples.append(str(e))
                continue
            if outcome is True:
                report.inserted += 1
            elif outcome is False:
                report.duplicates += 1
        return report

    def ingest_jsonl_file(self, path: str | Path) -> IngestReport:
        p = Path(path)
        if not p.is_file():
            return IngestReport()
        return self.ingest_jsonl_str(p.read_text(encoding="utf-8"))

    def query(self, q: EventQuery) -> list[StoredEvent]:
        sql = "SELECT id, timestamp_ms, event_type, channel, camera_name, confidence, classification_note, raw_json FROM events WHERE 1=1"
        args: list = []
        if q.camera_name is not None:
            sql += " AND camera_name = ?"
            args.append(q.camera_name)
        if q.event_type is not None:
            sql += " AND event_type = ?"
            args.append(q.event_type)
        if q.since_ms is not None:
            sql += " AND timestamp_ms >= ?"
            args.append(q.since_ms)
        if q.until_ms is not None:
            sql += " AND timestamp_ms <= ?"
            args.append(q.until_ms)
        if q.search_text is not None:
            sql += " AND raw_json LIKE ?"
            args.append(f"%{q.search_text}%")
        sql += " ORDER BY timestamp_ms DESC LIMIT ? OFFSET ?"
        args.extend([max(q.limit, 0), max(q.offset, 0)])
        rows = self._conn.execute(sql, args).fetchall()
        return [StoredEvent(*row) for row in rows]

    def device_activity(self) -> list[DeviceActivity]:
        rows = self._conn.execute(
            "SELECT camera_name, COUNT(*), MAX(timestamp_ms) FROM events "
            "WHERE camera_name IS NOT NULL GROUP BY camera_name "
            "ORDER BY MAX(timestamp_ms) DESC"
        ).fetchall()
        result = []
        for camera_name, count, last_seen in rows:
            types = [
                r[0]
                for r in self._conn.execute(
                    "SELECT DISTINCT event_type FROM events WHERE camera_name = ? ORDER BY event_type",
                    (camera_name,),
                ).fetchall()
            ]
            result.append(DeviceActivity(camera_name, count, types, last_seen))
        return result

    def analytics(self, now_ms: int) -> EventAnalytics:
        day_ms = 86_400_000
        today_start = (now_ms // day_ms) * day_ms
        events_today = self._conn.execute(
            "SELECT COUNT(*) FROM events WHERE timestamp_ms >= ?", (today_start,)
        ).fetchone()[0]
        by_camera = self._conn.execute(
            "SELECT camera_name, COUNT(*) FROM events WHERE camera_name IS NOT NULL "
            "GROUP BY camera_name ORDER BY COUNT(*) DESC"
        ).fetchall()
        by_type = self._conn.execute(
            "SELECT event_type, COUNT(*) FROM events GROUP BY event_type ORDER BY COUNT(*) DESC"
        ).fetchall()
        daily = []
        for day_offset in range(13, -1, -1):
            day_start = today_start - day_offset * day_ms
            day_end = day_start + day_ms
            count = self._conn.execute(
                "SELECT COUNT(*) FROM events WHERE timestamp_ms >= ? AND timestamp_ms < ?",
                (day_start, day_end),
            ).fetchone()[0]
            label = datetime.fromtimestamp(day_start / 1000, tz=timezone.utc).strftime(
                "%Y-%m-%d"
            )
            daily.append((label, count))
        return EventAnalytics(events_today, list(by_camera), list(by_type), daily)

    def apply_retention(
        self,
        now_ms: int,
        max_age_days: Optional[int] = None,
        max_rows: Optional[int] = None,
    ) -> int:
        deleted = 0
        if max_age_days is not None:
            cutoff = now_ms - max_age_days * 86_400_000
            cur = self._conn.execute("DELETE FROM events WHERE timestamp_ms < ?", (cutoff,))
            deleted += cur.rowcount
        if max_rows is not None:
            total = self.count_total()
            if total > max_rows:
                excess = total - max_rows
                cur = self._conn.execute(
                    "DELETE FROM events WHERE id IN "
                    "(SELECT id FROM events ORDER BY timestamp_ms ASC LIMIT ?)",
                    (excess,),
                )
                deleted += cur.rowcount
        self._conn.commit()
        return deleted

    def export_json(self, q: EventQuery) -> str:
        events = self.query(q)
        return json.dumps([vars(e) for e in events], ensure_ascii=False, indent=2)

    def export_csv(self, q: EventQuery) -> str:
        events = self.query(q)
        out = io.StringIO()
        writer = csv.writer(out)
        writer.writerow(["id", "timestamp_ms", "event_type", "channel", "camera_name", "confidence"])
        for e in events:
            writer.writerow([e.id, e.timestamp_ms, e.event_type, e.channel, e.camera_name or "", e.confidence])
        return out.getvalue()
