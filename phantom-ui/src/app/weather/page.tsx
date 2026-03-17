"use client";

import { useEffect, useState, useCallback } from "react";
import PhantomHeader from "@/components/PhantomHeader";
import NavPill from "@/components/NavPill";
import MiniCard from "@/components/MiniCard";
import ChaosTimeline from "@/components/ChaosTimeline";
import { api, AnalyzeResponse, PersistableSignal } from "@/lib/api";
import { DEFAULT_PARAMS } from "@/components/ParameterTuner";

interface MarketRowData {
  label: string;
  sublabel: string;
  prob: number;
  action: "ENTER" | "WATCH" | "SKIP";
  persisted: boolean;
}

function signalsToMarkets(signals: PersistableSignal[]): MarketRowData[] {
  return signals.map((s) => ({
    label: s.record.signal_type,
    sublabel: `${s.record.direction} · ${s.record.reason.slice(0, 40)}`,
    prob: Math.round(s.record.edge * 100),
    action: s.record.action,
    persisted: s.persisted,
  }));
}

export default function WeatherPage() {
  const [series, setSeries]     = useState<number[]>([]);
  const [results, setResults]   = useState<AnalyzeResponse[]>([]);
  const [markets, setMarkets]   = useState<MarketRowData[]>([]);
  const [signalsLoading, setSignalsLoading] = useState(true);
  const [loading, setLoading]   = useState(true);
  const [error, setError]       = useState<string | null>(null);

  const runAnalysis = useCallback(async (s: number[], signal: AbortSignal) => {
    const p   = DEFAULT_PARAMS;
    const win = 50;
    const step = Math.max(1, Math.floor((s.length - win) / 40));
    const windows: number[][] = [];
    for (let i = 0; i + win <= s.length; i += step) windows.push(s.slice(i, i + win));

    const analysisResults = await Promise.all(
      windows.map((w) => api.analyze({ series: w, dt: p.dt, dimension: p.dimension, tau: p.tau }))
    );
    if (!signal.aborted) setResults(analysisResults);
  }, []);

  useEffect(() => {
    const controller = new AbortController();

    async function load() {
      try {
        const [feed, signalsRes] = await Promise.allSettled([
          api.feeds.weather(40.7128, -74.006, 168),
          api.signals.analyze({ lat: 40.7128, lon: -74.006, weather_hours: 168 }),
        ]);

        if (controller.signal.aborted) return;

        if (feed.status === "fulfilled") {
          setSeries(feed.value.series);
          await runAnalysis(feed.value.series, controller.signal);
        } else {
          console.error("[WeatherPage] feed error:", feed.reason);
          setError(feed.reason instanceof Error ? feed.reason.message : "Failed to load feed");
        }

        if (signalsRes.status === "fulfilled") {
          const weatherSignals = signalsRes.value.signals.filter(
            (s) => s.record.market_type === "weather" || s.record.signal_type.toLowerCase().includes("weather")
          );
          setMarkets(signalsToMarkets(weatherSignals.length > 0 ? weatherSignals : signalsRes.value.signals));
        } else {
          console.error("[WeatherPage] signals error:", signalsRes.reason);
          setMarkets([]);
        }
      } catch (e) {
        if (!controller.signal.aborted) {
          console.error("[WeatherPage] load error:", e);
          setError(e instanceof Error ? e.message : "Failed to load");
        }
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false);
          setSignalsLoading(false);
        }
      }
    }

    load();
    return () => controller.abort();
  }, [runAnalysis]);

  const latest = results.at(-1);
  const opps   = results.filter((r) => r.chaos_score > 0.6).length;

  return (
    <div className="min-h-screen pb-28">
      <PhantomHeader
        section="WEATHER"
        subtitle={loading ? "fetching…" : "live"}
        status={loading ? "idle" : "live"}
        icon="cloud"
      />

      <main className="px-4 space-y-5">
        <div className="grid grid-cols-3 gap-3">
          <MiniCard label="Points" value={series.length ? `${series.length}` : "—"} />
          <MiniCard label="λ Max"  value={latest ? `${Math.max(0, latest.lambda).toFixed(2)}` : "—"} color="text-primary" />
          <MiniCard label="Opps"   value={`${opps.toString().padStart(2, "0")}`} />
        </div>

        <section className="glass-card rounded-[2rem] overflow-hidden">
          <div className="p-5 border-b" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
            <p className="label mb-1.5">Ensemble Divergence Scan</p>
            <div className="flex items-baseline gap-2">
              <span className="text-2xl font-light">
                {loading ? "Loading…"
                  : latest?.regime === "chaotic"      ? "Anomalous Flow"
                  : latest?.regime === "transitioning" ? "Regime Shift"
                  : latest                             ? "Stable Pattern"
                  : "No Data"}
              </span>
              {loading && <span className="w-2 h-2 rounded-full bg-primary animate-pulse" />}
            </div>
          </div>
          <div className="p-3" style={{ height: "224px" }}>
            {error ? (
              <div className="h-full flex items-center justify-center">
                <p className="font-mono text-xs text-alert">{error}</p>
              </div>
            ) : (
              <ChaosTimeline results={results} loading={loading} bare />
            )}
          </div>
        </section>

        <section className="space-y-3">
          <p className="label px-1">Active Kalshi Markets</p>
          {signalsLoading ? (
            <div className="glass-card rounded-2xl p-5 flex items-center justify-center">
              <span className="w-2 h-2 rounded-full bg-primary animate-pulse mr-2" />
              <span className="font-mono text-xs text-white/40">Fetching live signals…</span>
            </div>
          ) : markets.length === 0 ? (
            <div className="glass-card rounded-2xl p-5 text-center">
              <span className="font-mono text-xs text-white/30">No live signals available</span>
            </div>
          ) : (
            markets.map((m) => <MarketRow key={`${m.label}-${m.action}`} {...m} />)
          )}
        </section>
      </main>

      <NavPill />
    </div>
  );
}

function MarketRow({ label, sublabel, prob, action, persisted }: MarketRowData) {
  const isEnter  = action === "ENTER";
  const isWatch  = action === "WATCH";
  const accent   = isEnter
    ? "border-primary text-primary bg-primary/10 hover:bg-primary hover:text-black"
    : isWatch
    ? "border-solar text-solar bg-solar/10 hover:bg-solar hover:text-black"
    : "border-white/10 text-white/30 bg-white/5";
  const leftBorder = isEnter ? "border-l-primary" : isWatch ? "border-l-solar" : "border-l-white/10";

  return (
    <div className={`glass-card rounded-2xl p-5 flex items-center justify-between border-l-4 ${leftBorder}`}>
      <div className="flex flex-col gap-0.5">
        <div className="flex items-center gap-1.5">
          <span className="label">{sublabel}</span>
          {persisted && (
            <span
              className="w-1.5 h-1.5 rounded-full bg-primary/60 inline-block"
              title="Persisted to store"
            />
          )}
        </div>
        <span className="text-base font-light tracking-wide">{label}</span>
      </div>
      <div className="flex flex-col items-end gap-2">
        <span className={`font-mono text-xs ${isEnter ? "text-primary" : isWatch ? "text-solar" : "text-white/30"}`}>
          {prob}% EDGE
        </span>
        <button
          className={`px-4 py-1.5 rounded-full text-[9px] font-mono tracking-widest border transition-colors uppercase ${accent}`}
        >
          {action}
        </button>
      </div>
    </div>
  );
}
