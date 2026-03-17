"use client";

import { useEffect, useState, useCallback } from "react";
import PhantomHeader from "@/components/PhantomHeader";
import NavPill from "@/components/NavPill";
import MiniCard from "@/components/MiniCard";
import { api, SignalRecord } from "@/lib/api";

type FilterAction = "ALL" | "ENTER" | "WATCH" | "SKIP";

function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const s = Math.floor(diff / 1000);
  if (s < 60) return `${s}s ago`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.floor(h / 24);
  return `${d}d ago`;
}

export default function SignalsPage() {
  const [records, setRecords]   = useState<SignalRecord[]>([]);
  const [loading, setLoading]   = useState(true);
  const [running, setRunning]   = useState(false);
  const [filter, setFilter]     = useState<FilterAction>("ALL");
  const [error, setError]       = useState<string | null>(null);

  const fetchSignals = useCallback(async (action?: string) => {
    try {
      const res = await api.signals.query(action && action !== "ALL" ? { action, limit: 200 } : { limit: 200 });
      setRecords(res.records);
    } catch (e) {
      console.error("[SignalsPage] fetch error:", e);
      setError(e instanceof Error ? e.message : "Failed to load signals");
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();

    async function load() {
      try {
        await fetchSignals();
      } finally {
        if (!controller.signal.aborted) setLoading(false);
      }
    }

    load();
    return () => controller.abort();
  }, [fetchSignals]);

  async function handleRunAnalysis() {
    setRunning(true);
    setError(null);
    try {
      await api.signals.analyze();
      await fetchSignals(filter !== "ALL" ? filter : undefined);
    } catch (e) {
      console.error("[SignalsPage] analyze error:", e);
      setError(e instanceof Error ? e.message : "Analysis failed");
    } finally {
      setRunning(false);
    }
  }

  async function handleFilterChange(next: FilterAction) {
    setFilter(next);
    setLoading(true);
    try {
      await fetchSignals(next !== "ALL" ? next : undefined);
    } finally {
      setLoading(false);
    }
  }

  const enterCount = records.filter((r) => r.action === "ENTER").length;
  const watchCount = records.filter((r) => r.action === "WATCH").length;

  const displayed = filter === "ALL"
    ? records
    : records.filter((r) => r.action === filter);

  return (
    <div className="min-h-screen pb-28">
      <PhantomHeader
        section="SIGNALS"
        subtitle={loading ? "loading…" : running ? "analyzing…" : "live"}
        status={loading || running ? "idle" : "live"}
        icon="monitoring"
      />

      <main className="px-4 space-y-5">
        {/* Metric cards */}
        <div className="grid grid-cols-3 gap-3 fade-up fade-up-1">
          <MiniCard label="Total" value={records.length ? `${records.length}` : "—"} />
          <MiniCard
            label="Enter"
            value={enterCount ? `${enterCount}` : "—"}
            color="text-primary"
          />
          <MiniCard
            label="Watch"
            value={watchCount ? `${watchCount}` : "—"}
            color="text-solar"
          />
        </div>

        {/* Filter + Run Analysis bar */}
        <div className="flex items-center justify-between fade-up fade-up-2">
          <div className="flex items-center gap-1.5">
            {(["ALL", "ENTER", "WATCH", "SKIP"] as FilterAction[]).map((f) => {
              const active = filter === f;
              const activeColor =
                f === "ENTER" ? "border-primary text-primary bg-primary/10" :
                f === "WATCH" ? "border-solar text-solar bg-solar/10" :
                f === "SKIP"  ? "border-white/15 text-white/35 bg-white/5" :
                "border-white/20 text-white/60 bg-white/5";
              const inactiveColor = "border-white/10 text-white/25 hover:text-white/45 hover:border-white/20";
              return (
                <button
                  key={f}
                  onClick={() => handleFilterChange(f)}
                  className={`px-3 py-1 rounded-full font-mono text-[8px] tracking-widest uppercase border transition-all duration-200 ${
                    active ? activeColor : inactiveColor
                  }`}
                >
                  {f}
                </button>
              );
            })}
          </div>

          <button
            onClick={handleRunAnalysis}
            disabled={running}
            className="flex items-center gap-1.5 px-4 py-2 rounded-full border border-primary/40 text-primary bg-primary/8 hover:bg-primary/15 hover:border-primary/70 transition-all duration-200 disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <span
              className="material-symbols-outlined text-sm"
              style={{ fontVariationSettings: "'wght' 200, 'FILL' 0" }}
            >
              {running ? "sync" : "play_arrow"}
            </span>
            <span className="font-mono text-[8px] tracking-widest uppercase">
              {running ? "Running…" : "Run Analysis"}
            </span>
          </button>
        </div>

        {/* Error banner */}
        {error && (
          <div className="glass-card rounded-2xl p-4 border-alert/30 fade-up"
            style={{ borderColor: "rgba(255,51,51,0.3)" }}>
            <p className="font-mono text-[9px] text-alert tracking-widest">{error}</p>
          </div>
        )}

        {/* Signal list */}
        <section className="space-y-3 fade-up fade-up-3">
          <div className="flex items-center justify-between px-1">
            <p className="label">Signal History</p>
            <span className="font-mono text-[7px] text-white/20 tracking-widest">
              {displayed.length} RECORD{displayed.length !== 1 ? "S" : ""}
            </span>
          </div>

          {loading ? (
            <div className="glass-card rounded-2xl p-5">
              <span className="label">fetching signals…</span>
            </div>
          ) : displayed.length === 0 ? (
            <div className="glass-card rounded-2xl p-6 text-center">
              <span className="label">no signals found — run analysis to generate</span>
            </div>
          ) : (
            displayed.map((record, i) => (
              <SignalCard key={`${record.timestamp}-${i}`} record={record} />
            ))
          )}
        </section>
      </main>

      <NavPill />
    </div>
  );
}

function SignalCard({ record }: { record: SignalRecord }) {
  const isEnter = record.action === "ENTER";
  const isWatch = record.action === "WATCH";

  const accentHex  = isEnter ? "#00ff88" : isWatch ? "#ffbb00" : null;
  const accentCls  = isEnter ? "text-primary" : isWatch ? "text-solar" : "text-white/30";
  const badgeCls   = isEnter
    ? "border-primary/50 text-primary bg-primary/10"
    : isWatch
    ? "border-solar/50 text-solar bg-solar/10"
    : "border-white/10 text-white/30 bg-white/5";
  const borderStyle = accentHex
    ? { borderColor: `${accentHex}22` }
    : { borderColor: "rgba(255,255,255,0.06)" };

  const confColor =
    record.confidence === "HIGH"   ? "text-primary" :
    record.confidence === "MEDIUM" ? "text-solar"   :
    "text-white/30";

  return (
    <div
      className="glass-card rounded-2xl p-5 relative overflow-hidden"
      style={borderStyle}
    >
      {/* Ambient glow for actionable signals */}
      {isEnter && (
        <div className="absolute inset-0 rounded-2xl pointer-events-none"
          style={{ background: "radial-gradient(ellipse at left center, rgba(0,255,136,0.07) 0%, transparent 60%)" }} />
      )}
      {isWatch && (
        <div className="absolute inset-0 rounded-2xl pointer-events-none"
          style={{ background: "radial-gradient(ellipse at left center, rgba(255,187,0,0.06) 0%, transparent 60%)" }} />
      )}

      <div className="relative flex items-start justify-between gap-3">
        {/* Left: signal type + reason */}
        <div className="flex flex-col gap-2 min-w-0 flex-1">
          <div className="flex items-center gap-2">
            {/* Status dot */}
            {accentHex ? (
              <span className="w-1.5 h-1.5 rounded-full shrink-0" style={{ backgroundColor: accentHex }} />
            ) : (
              <span className="w-1.5 h-1.5 rounded-full bg-white/20 shrink-0" />
            )}
            <span className={`font-mono text-[9px] tracking-widest uppercase ${accentCls}`}>
              {record.signal_type}
            </span>
          </div>

          <p className="text-sm font-light leading-snug text-white/70 line-clamp-2">
            {record.reason}
          </p>

          {/* Bottom row: direction + confidence + chaos */}
          <div className="flex items-center gap-3 mt-0.5">
            <span className="font-mono text-[8px] tracking-widest text-white/35">
              DIR: <span className="text-white/55">{record.direction}</span>
            </span>
            <span className={`font-mono text-[8px] tracking-widest ${confColor}`}>
              CONF: {record.confidence}
            </span>
            <span className="font-mono text-[8px] tracking-widest text-white/35">
              CHAOS: <span className="text-white/55">{record.chaos_score.toFixed(2)}</span>
            </span>
          </div>
        </div>

        {/* Right: action badge + edge + time */}
        <div className="flex flex-col items-end gap-2 shrink-0">
          <span className={`font-mono text-[8px] tracking-widest border px-2.5 py-1 rounded-full uppercase ${badgeCls}`}>
            {record.action}
          </span>
          <span className={`font-mono text-sm font-light ${accentCls}`}>
            {record.edge >= 0 ? "+" : ""}{(record.edge * 100).toFixed(1)}
            <span className="text-[8px] text-white/30 ml-0.5">%</span>
          </span>
          <span className="font-mono text-[7px] tracking-widest text-white/25">
            {relativeTime(record.timestamp)}
          </span>
        </div>
      </div>
    </div>
  );
}
