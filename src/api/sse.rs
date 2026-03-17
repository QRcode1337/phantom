//! Server-Sent Events endpoint for live signal streaming.
//!
//! Route (registered in api/mod.rs):
//!   GET /api/signals/stream — SSE stream of new signals + heartbeat

use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::{Stream, StreamExt};
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use super::signals_db::SignalRecord;

// ─── Broadcast channel ───────────────────────────────────────────────────────

lazy_static::lazy_static! {
    static ref SIGNAL_TX: broadcast::Sender<SignalRecord> = {
        let (tx, _) = broadcast::channel(256);
        tx
    };
}

/// Broadcast a signal to all connected SSE clients.
///
/// Fire-and-forget: if no clients are subscribed the signal is silently dropped.
pub fn broadcast_signal(record: &SignalRecord) {
    let _ = SIGNAL_TX.send(record.clone());
}

// ─── GET /api/signals/stream ─────────────────────────────────────────────────

pub async fn signal_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = SIGNAL_TX.subscribe();
    let signal_stream = BroadcastStream::new(rx).filter_map(|result| {
        let event = match result {
            Ok(record) => {
                serde_json::to_string(&record)
                    .ok()
                    .map(|json| Ok(Event::default().event("signal").data(json)))
            }
            // Lagged receivers lose messages; skip rather than error.
            Err(_) => None,
        };
        std::future::ready(event)
    });

    let heartbeat = tokio_stream::StreamExt::map(
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(15))),
        |_| Ok(Event::default().event("heartbeat").data("ping")),
    );

    let merged = futures_util::stream::select(signal_stream, heartbeat);

    Sse::new(merged).keep_alive(KeepAlive::default())
}
