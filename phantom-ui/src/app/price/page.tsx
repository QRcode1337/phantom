"use client";

import { useEffect, useState } from "react";
import PhantomHeader from "@/components/PhantomHeader";
import NavPill from "@/components/NavPill";
import MiniCard from "@/components/MiniCard";
import PhaseSpace from "@/components/PhaseSpace";
import ChaosTimeline from "@/components/ChaosTimeline";
import { api, AnalyzeResponse, EmbedResponse, PersistableSignal } from "@/lib/api";
import { DEFAULT_PARAMS } from "@/components/ParameterTuner";

interface RegimeRow { ticker: string; price: string; score?: number; regime?: string; tag: string; }

function signalToTag(s: PersistableSignal): string {
  if (s.record.action === "ENTER" && s.record.chaos_score > 0.7) return "BREAKOUT";
  if (s.record.action === "ENTER") return "ENTER";
  if (s.record.chaos_score > 0.7) return "BREAKOUT";
  if (s.record.chaos_score > 0.4) return "TRENDING";
  return "LATERAL";
}

export default function PricePage() {
  const [series, setSeries]   = useState<number[]>([]);
  const [results, setResults] = useState<AnalyzeResponse[]>([]);
  const [embed, setEmbed]     = useState<EmbedResponse | undefined>();
  const [loading, setLoading] = useState(true);
  const [btcScore, setBtcScore]  = useState<number | undefined>();
  const [btcRegime, setBtcRegime] = useState<string | undefined>();
  const [liveRows, setLiveRows]   = useState<RegimeRow[]>([]);

  useEffect(() => {
    const controller = new AbortController();

    async function load() {
      try {
        const [feedRes, signalsRes] = await Promise.allSettled([
          api.feeds.btc(30),
          api.signals.analyze({ btc_days: 30 }),
        ]);

        if (controller.signal.aborted) return;

        if (feedRes.status === "fulfilled") {
          const feed = feedRes.value;
          setSeries(feed.series);
          const p = DEFAULT_PARAMS;

          const [embedRes, analyzeRes] = await Promise.all([
            api.embed({ series: feed.series, dimension: p.dimension, tau: p.tau }),
            api.analyze({ series: feed.series, dt: p.dt, dimension: p.dimension, tau: p.tau }),
          ]);
          if (controller.signal.aborted) return;

          setEmbed(embedRes);
          setBtcScore(analyzeRes.chaos_score);
          setBtcRegime(analyzeRes.regime);

          const win  = 50;
          const step = Math.max(1, Math.floor((feed.series.length - win) / 25));
          const windows: number[][] = [];
          for (let i = 0; i + win <= feed.series.length; i += step) {
            windows.push(feed.series.slice(i, i + win));
          }
          const settled = await Promise.allSettled(
            windows.map((w) => api.analyze({ series: w, dt: p.dt, dimension: p.dimension, tau: p.tau }))
          );
          const timelineResults = settled
            .flatMap((r) => r.status === "fulfilled" ? [r.value] : []);
          if (!controller.signal.aborted) setResults(timelineResults);
        } else {
          console.error("[PricePage] feed error:", feedRes.reason);
        }

        if (signalsRes.status === "fulfilled") {
          const priceSignals = signalsRes.value.signals.filter(
            (s) => s.record.market_type === "price" || s.record.signal_type.toLowerCase().includes("btc") ||
                   s.record.signal_type.toLowerCase().includes("price")
          );
          const rows = priceSignals.map((s) => ({
            ticker: s.record.signal_type,
            price: "—",
            score: s.record.chaos_score,
            regime: s.record.chaos_score > 0.7 ? "chaotic" : s.record.chaos_score > 0.4 ? "transitioning" : "stable",
            tag: signalToTag(s),
          }));
          if (!controller.signal.aborted) setLiveRows(rows);
        } else {
          console.error("[PricePage] signals error:", signalsRes.reason);
        }
      } catch (e) {
        if (!controller.signal.aborted) {
          console.error("[PricePage] load error:", e);
        }
      } finally {
        if (!controller.signal.aborted) setLoading(false);
      }
    }

    load();
    return () => controller.abort();
  }, []);

  const btcRow: RegimeRow = {
    ticker: "BTC / USD",
    price: series.at(-1)?.toFixed(0) ?? "—",
    score: btcScore,
    regime: btcRegime,
    tag: btcRegime === "chaotic" ? "BREAKOUT" : "TRENDING",
  };

  const comingSoonRows: RegimeRow[] = [
    { ticker: "ETH / USD", price: "—", tag: "SOON" },
    { ticker: "SOL / USD", price: "—", tag: "SOON" },
  ];

  const rows: RegimeRow[] = [
    btcRow,
    ...liveRows,
    ...comingSoonRows,
  ];

  const chaotic = rows.filter((r) => r.regime === "chaotic").length;

  return (
    <div className="min-h-screen pb-28">
      <PhantomHeader
        section="PRICE"
        subtitle={loading ? "loading…" : "live"}
        status={loading ? "idle" : "live"}
        icon="show_chart"
      />

      <main className="px-4 space-y-5">
        <div className="grid grid-cols-3 gap-3 fade-up fade-up-1">
          <MiniCard label="Tracked" value="24" />
          <MiniCard label="Volatile" value={chaotic > 0 ? "HIGH" : "LOW"} color={chaotic > 0 ? "text-alert" : "text-primary"} />
          <MiniCard label="Regimes" value={`0${rows.filter((r) => r.regime).length}`} color="text-primary" />
        </div>

        <section className="glass-card rounded-[2rem] overflow-hidden fade-up fade-up-2">
          <div className="p-5 border-b" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
            <p className="label mb-1.5">Chaos Phase Space Reconstruction</p>
            <div className="w-10 h-px" style={{ background: "linear-gradient(90deg, rgba(0,255,136,0.6), transparent)" }} />
          </div>
          <div style={{ height: "260px" }} className="p-2 relative">
            <PhaseSpace embed={embed} loading={loading} bare />
            <div className="absolute bottom-3 right-4 text-right">
              <p className="font-mono text-[7px] text-white/20 tracking-widest">ENGINE: NOMINAL</p>
              <p className="font-mono text-[7px] text-white/20 tracking-widest">
                DIM: {DEFAULT_PARAMS.dimension} // TAU: {DEFAULT_PARAMS.tau}
              </p>
            </div>
          </div>
        </section>

        <section className="glass-card rounded-[2rem] overflow-hidden fade-up fade-up-3">
          <div className="p-5 border-b" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
            <p className="label mb-1">Regime Timeline — BTC 30d</p>
          </div>
          <div style={{ height: "180px" }} className="p-2">
            <ChaosTimeline results={results} loading={loading} bare />
          </div>
        </section>

        <section className="space-y-3 fade-up fade-up-4">
          <p className="label px-1">Active Regime Monitoring</p>
          {rows.map((row) => <RegimeRowCard key={row.ticker} {...row} />)}
        </section>
      </main>

      <NavPill />
    </div>
  );
}

function RegimeRowCard({ ticker, price, score, regime, tag }: RegimeRow) {
  const isSoon     = tag === "SOON";
  const isChaotic  = regime === "chaotic" && !isSoon;
  const accentColor    = isChaotic ? "#ffbb00" : undefined;
  const scoreColor     = isChaotic ? "text-solar" : regime === "transitioning" ? "text-solar" : "text-primary";
  const borderColor    = isChaotic
    ? "rgba(255,187,0,0.3)"
    : isSoon
    ? "rgba(255,255,255,0.04)"
    : "rgba(255,255,255,0.06)";

  return (
    <div
      className={`glass-card rounded-2xl p-5 flex items-center justify-between relative${isSoon ? " opacity-40" : ""}`}
      style={{ borderColor }}
    >
      {isChaotic && (
        <div className="absolute inset-0 rounded-2xl"
          style={{ background: "radial-gradient(ellipse at left, rgba(255,187,0,0.06) 0%, transparent 60%)" }} />
      )}
      <div className="relative flex flex-col gap-1">
        <span className="font-mono text-[9px] tracking-widest uppercase"
          style={{ color: accentColor ?? "rgba(255,255,255,0.35)" }}>
          {ticker}
        </span>
        <span className="text-xl font-light">{price !== "—" ? `$${price}` : "—"}</span>
      </div>
      <div className="relative flex flex-col items-end gap-1">
        <span className="label">Chaos Score</span>
        <span className={`font-mono text-sm ${isSoon ? "text-white/20" : scoreColor}`}>
          {isSoon ? "COMING SOON" : score !== undefined ? `${score.toFixed(2)} // ${regime?.toUpperCase()}` : "— // —"}
        </span>
      </div>
      <div className="relative w-px h-8 mx-2" style={{ background: "rgba(255,255,255,0.08)" }} />
      <span className="relative font-mono text-[9px] tracking-widest"
        style={{ color: accentColor ?? "rgba(255,255,255,0.25)" }}>
        {tag}
      </span>
    </div>
  );
}
