"use client";

import { useState } from "react";
import { api } from "@/lib/api";

interface Props {
  onData: (series: number[]) => void;
}

type FeedKey = "weather" | "seismic" | "btc";

const FEEDS: Record<FeedKey, string> = {
  weather: "Weather",
  seismic: "Seismic",
  btc:     "BTC 30d",
};

function parseSeries(raw: string): number[] {
  return raw.split(/[\s,;]+/).map((s) => s.trim()).filter(Boolean)
    .map(Number).filter((n) => !isNaN(n));
}

function stats(s: number[]) {
  if (!s.length) return null;
  return {
    count: s.length,
    min:   Math.min(...s).toFixed(3),
    max:   Math.max(...s).toFixed(3),
    mean:  (s.reduce((a, b) => a + b, 0) / s.length).toFixed(3),
  };
}

export default function DataLoader({ onData }: Props) {
  const [text, setText] = useState("");
  const [series, setSeries] = useState<number[]>([]);
  const [loading, setLoading] = useState<FeedKey | null>(null);
  const [error, setError] = useState<string | null>(null);

  function handleLoad() {
    const parsed = parseSeries(text);
    if (!parsed.length) { setError("No valid numbers."); return; }
    setError(null);
    setSeries(parsed);
    onData(parsed);
  }

  async function handleFeed(feed: FeedKey) {
    setLoading(feed);
    setError(null);
    try {
      let result;
      if (feed === "weather") result = await api.feeds.weather(40.7128, -74.006, 168);
      else if (feed === "seismic") result = await api.feeds.seismic("all_day");
      else result = await api.feeds.btc(30);
      setSeries(result.series);
      onData(result.series);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Fetch failed");
    } finally {
      setLoading(null);
    }
  }

  const s = stats(series);

  return (
    <div className="glass-card rounded-3xl p-4 flex flex-col gap-3">
      <p className="label">Data Input</p>

      <textarea
        value={text}
        onChange={(e) => setText(e.target.value)}
        rows={3}
        placeholder="Paste numbers…"
        className="w-full rounded-xl px-3 py-2 font-mono text-[10px] text-white/70 resize-none focus:outline-none"
        style={{ background: "rgba(255,255,255,0.04)", border: "0.5px solid rgba(255,255,255,0.08)" }}
      />
      <button onClick={handleLoad}
        className="self-start px-3 py-1.5 rounded-full font-mono text-[9px] tracking-widest transition-all"
        style={{ background: "rgba(255,255,255,0.06)", border: "0.5px solid rgba(255,255,255,0.12)", color: "rgba(255,255,255,0.6)" }}>
        LOAD
      </button>

      <div className="flex flex-col gap-1.5">
        <p className="label">Live Feeds</p>
        <div className="flex gap-2 flex-wrap">
          {(Object.keys(FEEDS) as FeedKey[]).map((k) => (
            <button key={k} onClick={() => handleFeed(k)} disabled={loading !== null}
              className="px-3 py-1.5 rounded-full font-mono text-[9px] tracking-widest transition-all disabled:opacity-40"
              style={{ background: "rgba(0,255,136,0.08)", border: "0.5px solid rgba(0,255,136,0.2)", color: "rgba(0,255,136,0.8)" }}>
              {loading === k ? "…" : FEEDS[k]}
            </button>
          ))}
        </div>
      </div>

      {s && (
        <div className="grid grid-cols-2 gap-1.5">
          {[["n", s.count], ["min", s.min], ["max", s.max], ["mean", s.mean]].map(([k, v]) => (
            <div key={k as string} className="rounded-lg px-2 py-1.5 flex justify-between"
              style={{ background: "rgba(255,255,255,0.03)" }}>
              <span className="font-mono text-[8px] text-white/25 tracking-widest uppercase">{k}</span>
              <span className="font-mono text-[9px] text-white/60">{v}</span>
            </div>
          ))}
        </div>
      )}

      {error && <p className="font-mono text-[9px] text-alert">{error}</p>}
    </div>
  );
}
