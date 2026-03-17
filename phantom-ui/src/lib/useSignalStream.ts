"use client";

import { useState, useEffect, useRef } from "react";
import { SignalRecord } from "./api";

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
const MAX_SIGNALS = 50;
const RECONNECT_DELAY = 3000;

export type StreamStatus = "connected" | "disconnected" | "error";

export interface SignalStreamResult {
  signals: SignalRecord[];
  status: StreamStatus;
  lastHeartbeat: Date | null;
  /** ISO timestamp of the most recent signal received via SSE, or null */
  latestSignalTimestamp: string | null;
}

export function useSignalStream(): SignalStreamResult {
  const [signals, setSignals] = useState<SignalRecord[]>([]);
  const [status, setStatus] = useState<StreamStatus>("disconnected");
  const [lastHeartbeat, setLastHeartbeat] = useState<Date | null>(null);
  const [latestSignalTimestamp, setLatestSignalTimestamp] = useState<string | null>(null);

  const esRef = useRef<EventSource | null>(null);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;

    function connect() {
      if (!mountedRef.current) return;

      // Clean up any existing connection
      if (esRef.current) {
        esRef.current.close();
        esRef.current = null;
      }

      const es = new EventSource(`${API_BASE}/api/signals/stream`);
      esRef.current = es;

      es.addEventListener("open", () => {
        if (!mountedRef.current) return;
        setStatus("connected");
      });

      es.addEventListener("signal", (event: MessageEvent) => {
        if (!mountedRef.current) return;
        try {
          const record = JSON.parse(event.data) as SignalRecord;
          setSignals((prev) => [record, ...prev].slice(0, MAX_SIGNALS));
          setLatestSignalTimestamp(record.timestamp);
        } catch (e) {
          console.error("[useSignalStream] failed to parse signal event:", e);
        }
      });

      es.addEventListener("heartbeat", () => {
        if (!mountedRef.current) return;
        setLastHeartbeat(new Date());
      });

      es.addEventListener("error", () => {
        if (!mountedRef.current) return;
        setStatus("error");
        es.close();
        esRef.current = null;

        // Auto-reconnect after 3 seconds
        reconnectTimerRef.current = setTimeout(() => {
          if (mountedRef.current) {
            setStatus("disconnected");
            connect();
          }
        }, RECONNECT_DELAY);
      });
    }

    connect();

    return () => {
      mountedRef.current = false;

      if (reconnectTimerRef.current !== null) {
        clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }

      if (esRef.current) {
        esRef.current.close();
        esRef.current = null;
      }
    };
  }, []);

  return { signals, status, lastHeartbeat, latestSignalTimestamp };
}
