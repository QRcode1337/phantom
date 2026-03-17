//! JSON-lines signal persistence for PHANTOM backtesting.
//!
//! Storage format: one JSON object per line in `signals.jsonl`.
//! Path is configurable via the `PHANTOM_SIGNALS_PATH` env var;
//! defaults to `./signals.jsonl` relative to the process working directory.
//!
//! No external database dependency — relies only on `serde_json`, `chrono`,
//! and `std::fs` which are already available in this crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

// ─── Domain types ─────────────────────────────────────────────────────────────

/// Result of a resolved signal for backtesting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutcomeResult {
    Win,
    Loss,
    Scratch,  // breakeven
    Expired,  // market expired without entry
}

/// Outcome data attached to a signal after resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalOutcome {
    pub resolved_at: DateTime<Utc>,
    pub result: OutcomeResult,
    pub pnl: Option<f64>,
    pub notes: Option<String>,
}

/// A single persisted signal record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRecord {
    /// ISO-8601 UTC timestamp when the signal was generated.
    pub timestamp: DateTime<Utc>,
    /// Feed or analysis type, e.g. "weather/temperature", "price/BTC-USD",
    /// "weather/regime-shift", or "manual".
    pub signal_type: String,
    /// "ENTER" | "WATCH" | "SKIP"
    pub action: String,
    /// Measured edge probability advantage (0.0–0.5).
    pub edge: f64,
    /// FTLE chaos score in the underlying series (0.0–1.0).
    pub chaos_score: f64,
    /// Market category, e.g. "weather", "price", "seismic".
    pub market_type: String,
    /// Trade direction: "YES" or "NO".
    pub direction: String,
    /// "HIGH" | "MEDIUM" | "LOW"
    pub confidence: String,
    /// Human-readable rationale for the signal.
    pub reason: String,
    /// Outcome fields — filled in after signal resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<SignalOutcome>,
}

// ─── Store ────────────────────────────────────────────────────────────────────

/// Append-only JSON-lines file store for signal records.
pub struct SignalStore {
    path: PathBuf,
}

impl SignalStore {
    /// Create a new `SignalStore`.
    /// Reads `PHANTOM_SIGNALS_PATH` from the environment; falls back to
    /// `./signals.jsonl`.
    pub fn new() -> Self {
        let path = std::env::var("PHANTOM_SIGNALS_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("signals.jsonl"));
        Self { path }
    }

    /// Append a single record to the backing file.
    ///
    /// Each call opens, writes one newline-terminated JSON object, and closes
    /// the file — safe for concurrent callers as long as each write is smaller
    /// than the OS pipe buffer (~64 KiB on Linux/macOS), which is always true
    /// for signal records.
    pub fn store_signal(&self, record: &SignalRecord) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let line = serde_json::to_string(record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        writeln!(file, "{}", line)
    }

    /// Load all records from the file.
    /// Lines that fail to parse are silently skipped so a corrupt entry
    /// never breaks a full read.
    pub fn load_signals(&self) -> std::io::Result<Vec<SignalRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let records = reader
            .lines()
            .filter_map(|line| {
                let line = line.ok()?;
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }
                serde_json::from_str::<SignalRecord>(trimmed).ok()
            })
            .collect();

        Ok(records)
    }

    /// Load only records whose `timestamp` is >= `since`.
    pub fn load_signals_since(&self, since: DateTime<Utc>) -> std::io::Result<Vec<SignalRecord>> {
        Ok(self
            .load_signals()?
            .into_iter()
            .filter(|r| r.timestamp >= since)
            .collect())
    }

    /// Return the total number of stored records (fast linear scan; no index).
    pub fn count(&self) -> std::io::Result<usize> {
        if !self.path.exists() {
            return Ok(0);
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let count = reader
            .lines()
            .filter(|l| l.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false))
            .count();

        Ok(count)
    }

    /// Resolve a signal by matching its timestamp, attaching the given outcome.
    ///
    /// Reads all records, updates the first match, and rewrites the file.
    /// Returns `true` if a matching record was found and updated.
    pub fn resolve_signal(
        &self,
        timestamp: DateTime<Utc>,
        outcome: SignalOutcome,
    ) -> std::io::Result<bool> {
        let mut records = self.load_signals()?;

        let found = records
            .iter_mut()
            .find(|r| r.timestamp == timestamp);

        match found {
            Some(record) => {
                record.outcome = Some(outcome);
                self.rewrite_all(&records)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Overwrite the backing file with the given records.
    fn rewrite_all(&self, records: &[SignalRecord]) -> std::io::Result<()> {
        let mut file = File::create(&self.path)?;
        for record in records {
            let line = serde_json::to_string(record)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }
}

impl Default for SignalStore {
    fn default() -> Self {
        Self::new()
    }
}
