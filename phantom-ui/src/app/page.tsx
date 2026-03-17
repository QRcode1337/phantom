"use client";

import { useEffect, useState } from "react";
import PhantomHeader from "@/components/PhantomHeader";
import NavPill from "@/components/NavPill";
import { api, HealthResponse, AnalyzeSignalsResponse, SignalRecord } from "@/lib/api";
import { useSignalStream } from "@/lib/useSignalStream";

export default function OverviewPage() {
  const [health, setHealth]   = useState<HealthResponse | null>(null);
  const [signals, setSignals] = useState<AnalyzeSignalsResponse | null>(null);
  const [signalsLoading, setSignalsLoading] = useState(true);
  // Live chaos score override from SSE — supersedes the batch analyze result
  const [liveChaos, setLiveChaos] = useState<number | undefined>(undefined);
  // Per-feed action overrides from SSE signals
  const [liveActions, setLiveActions] = useState<Record<string, string>>({});

  const { signals: sseSignals, latestSignalTimestamp } = useSignalStream();

  // Update live state whenever a new SSE signal arrives
  useEffect(() => {
    if (sseSignals.length === 0) return;
    const newest: SignalRecord = sseSignals[0];

    // Update chaos score if higher than current live value
    setLiveChaos((prev) =>
      prev === undefined || newest.chaos_score > prev ? newest.chaos_score : prev
    );

    // Update the action for the matching feed key
    const sigType = newest.signal_type.toLowerCase();
    const mktType = newest.market_type.toLowerCase();
    const feedKey =
      sigType.includes("weather") || mktType.includes("weather") ? "weather" :
      sigType.includes("btc") || mktType.includes("btc")         ? "btc"     :
      sigType.includes("seismic") || mktType.includes("seismic") ? "seismic" :
      null;

    if (feedKey) {
      setLiveActions((prev) => ({ ...prev, [feedKey]: newest.action }));
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [latestSignalTimestamp]);

  useEffect(() => {
    const controller = new AbortController();

    Promise.allSettled([
      api.feeds.health(),
      api.signals.analyze(),
    ]).then(([healthRes, signalsRes]) => {
      if (controller.signal.aborted) return;
      if (healthRes.status === "fulfilled") setHealth(healthRes.value);
      if (signalsRes.status === "fulfilled") setSignals(signalsRes.value);
      setSignalsLoading(false);
    }).catch(() => {
      if (!controller.signal.aborted) setSignalsLoading(false);
    });

    return () => controller.abort();
  }, []);

  const feedCount = health ? Object.keys(health.feeds).length : 0;

  // Derive top-level chaos score: prefer live SSE value, fall back to batch result
  const batchChaos = signals?.signals.length
    ? Math.max(...signals.signals.map((s) => s.record.chaos_score))
    : undefined;
  const latestChaos = liveChaos ?? batchChaos;
  const chaosDisplay = signalsLoading ? "…" : latestChaos !== undefined ? latestChaos.toFixed(2) : "—";
  const chaosColor   = latestChaos !== undefined
    ? latestChaos > 0.7 ? "text-alert" : latestChaos > 0.4 ? "text-solar" : "text-primary"
    : "text-primary";

  // Derive per-feed actions: prefer live SSE overrides, fall back to batch
  function feedAction(signalType: string, healthKey: string): string {
    if (health?.feeds[healthKey] !== "available") return "OFFLINE";
    // Live SSE override takes precedence
    if (liveActions[healthKey]) return liveActions[healthKey];
    if (!signals || signalsLoading) return "WATCH";
    const match = signals.signals.find(
      (s) => s.record.signal_type.toLowerCase().includes(signalType) ||
             s.record.market_type.toLowerCase().includes(signalType)
    );
    return match?.record.action ?? "WATCH";
  }

  const weatherAction = feedAction("weather", "weather");
  const priceAction   = feedAction("btc", "btc");
  const seismicAction = feedAction("seismic", "seismic");

  return (
    <div className="min-h-screen pb-28">
      <PhantomHeader section="ENGINE" subtitle="Tactical OS v3.0" status="live" icon="hub" />

      <main className="px-4 space-y-5">
        {/* Metrics row */}
        <div className="grid grid-cols-3 gap-3">
          <MetricCard icon="radar" label="Feeds" value={health ? `${feedCount}` : "—"} />
          <MetricCard icon="blur_on" label="Chaos" value={chaosDisplay} color={chaosColor} />
          <MetricCard icon="speed" label="Sync" value="<10ms" color="text-primary" />
        </div>

        {/* Main attractor visualization */}
        <section className="glass-card rounded-[2rem] overflow-hidden">
          <div className="p-5 border-b border-white/5 flex justify-between items-end">
            <div>
              <p className="label mb-2">Chaos Core Engine</p>
              <div className="flex items-baseline gap-2">
                <span className="text-2xl hairline">Phase Space</span>
                <span className="text-primary font-mono text-xs">ACTIVE</span>
              </div>
            </div>
            <span className="material-symbols-outlined text-white/10 text-5xl"
              style={{ fontVariationSettings: "'wght' 100" }}>
              blur_circular
            </span>
          </div>

          {/* Lorenz attractor SVG */}
          <div className="p-6 ribbon-3d">
            <svg viewBox="0 0 300 180" className="w-full" style={{ opacity: 0.75 }}>
              <defs>
                <linearGradient id="lorenzGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stopColor="#00ff88" stopOpacity="0.9" />
                  <stop offset="40%" stopColor="#0088ff" stopOpacity="0.5" />
                  <stop offset="100%" stopColor="#ff00ff" stopOpacity="0.3" />
                </linearGradient>
                <filter id="glow">
                  <feGaussianBlur stdDeviation="2" result="coloredBlur" />
                  <feMerge><feMergeNode in="coloredBlur" /><feMergeNode in="SourceGraphic" /></feMerge>
                </filter>
              </defs>
              {/* Wing 1 */}
              <path d="M150,90 C200,30 270,60 240,110 C210,155 150,140 150,90"
                fill="none" stroke="url(#lorenzGrad)" strokeWidth="0.8" filter="url(#glow)" />
              <path d="M150,90 C100,30 30,60 60,110 C90,155 150,140 150,90"
                fill="none" stroke="url(#lorenzGrad)" strokeWidth="0.8" filter="url(#glow)" opacity="0.7" />
              {/* Inner loops */}
              <path d="M150,90 C185,55 230,75 215,105 C200,130 155,125 150,90"
                fill="none" stroke="#00ff88" strokeWidth="0.4" opacity="0.4" />
              <path d="M150,90 C115,55 70,75 85,105 C100,130 145,125 150,90"
                fill="none" stroke="#00ff88" strokeWidth="0.4" opacity="0.4" />
              {/* Orbit dots */}
              <circle cx="240" cy="110" r="2" fill="#00ff88" opacity="0.8" filter="url(#glow)" />
              <circle cx="60" cy="110" r="1.5" fill="#ff00ff" opacity="0.6" filter="url(#glow)" />
              <circle cx="150" cy="90" r="1" fill="white" opacity="0.5" />
              {/* Grid lines */}
              <line x1="0" y1="90" x2="300" y2="90" stroke="white" strokeWidth="0.2" opacity="0.1" />
              <line x1="150" y1="0" x2="150" y2="180" stroke="white" strokeWidth="0.2" opacity="0.1" />
              {/* Dashed trajectory */}
              <path d="M150,90 C200,30 270,60 240,110 C210,155 150,140 100,130 C50,120 30,80 60,55 C90,30 150,45 200,75"
                fill="none" stroke="white" strokeWidth="0.3" strokeDasharray="3 4" opacity="0.15" />
            </svg>
          </div>

          {/* Engine status list */}
          <div className="divide-y divide-white/5">
            <FeedRow icon="cloud" label="Weather" sublabel="Open-Meteo // NYC"
              status={weatherAction} active={weatherAction === "ENTER"} />
            <FeedRow icon="show_chart" label="Price" sublabel="CoinGecko // BTC"
              status={priceAction} />
            <FeedRow icon="radar" label="Argus" sublabel="USGS // Seismic"
              status={seismicAction} />
          </div>
        </section>
      </main>

      <NavPill />
    </div>
  );
}

function MetricCard({ icon, label, value, color = "text-white/90" }: {
  icon: string; label: string; value: string; color?: string;
}) {
  return (
    <div className="glass-card p-4 rounded-3xl relative overflow-hidden">
      <div className="absolute -right-1 -top-1 opacity-8">
        <span className="material-symbols-outlined text-4xl text-white/10"
          style={{ fontVariationSettings: "'wght' 100" }}>{icon}</span>
      </div>
      <p className="label mb-1.5">{label}</p>
      <p className={`font-mono text-xl hairline ${color}`}>{value}</p>
    </div>
  );
}

function FeedRow({ icon, label, sublabel, status, active }: {
  icon: string; label: string; sublabel: string; status: string; active?: boolean;
}) {
  const isEnter = status === "ENTER";
  const isWatch = status === "WATCH";
  return (
    <div className="p-5 flex items-center justify-between relative overflow-hidden group">
      {active && <div className="absolute inset-0 aurora-green opacity-50" />}
      <div className="relative z-10 flex items-center gap-4">
        <span className="material-symbols-outlined text-white/30 text-2xl"
          style={{ fontVariationSettings: "'wght' 200" }}>{icon}</span>
        <div className="flex flex-col">
          <span className="text-base hairline tracking-wider">{label}</span>
          <span className="font-mono text-[9px] opacity-40 tracking-widest">{sublabel}</span>
        </div>
      </div>
      <div className="relative z-10 flex flex-col items-end gap-1">
        <span className={`font-mono text-xs ${isEnter ? "text-primary" : isWatch ? "text-solar" : "text-white/30"}`}>
          {status}
        </span>
        <div className={`h-px ${isEnter ? "w-10 bg-primary/40" : isWatch ? "w-7 bg-solar/40" : "w-4 bg-white/10"}`} />
      </div>
    </div>
  );
}
