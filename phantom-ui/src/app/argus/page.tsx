"use client";

import { useEffect, useState } from "react";
import PhantomHeader from "@/components/PhantomHeader";
import NavPill from "@/components/NavPill";
import MiniCard from "@/components/MiniCard";
import FtleHeatmap from "@/components/FtleHeatmap";
import { api, FtleFieldResponse } from "@/lib/api";
import { DEFAULT_PARAMS } from "@/components/ParameterTuner";

interface AnomalyEvent {
  id: string;
  title: string;
  sub: string;
  severity: "critical" | "warning" | "nominal";
  time: string;
}

export default function ArgusPage() {
  const [ftle, setFtle]               = useState<FtleFieldResponse | undefined>();
  const [loading, setLoading]         = useState(true);
  const [seismicCount, setSeismicCount] = useState(0);
  const [flightCount, setFlightCount]   = useState(0);
  const [critCount, setCritCount]       = useState(0);
  const [events, setEvents]             = useState<AnomalyEvent[]>([]);

  useEffect(() => {
    const controller = new AbortController();

    async function load() {
      try {
        const [seismic, opensky] = await Promise.all([
          api.feeds.seismic("all_day"),
          api.feeds.opensky().catch((e) => {
            console.warn("[ArgusPage] opensky unavailable:", e);
            return { states: [], count: 0 };
          }),
        ]);
        if (controller.signal.aborted) return;

        setSeismicCount(seismic.series.length);
        setFlightCount(opensky.count);

        if (seismic.series.length >= 50) {
          const p = DEFAULT_PARAMS;
          const winSize = Math.max(50, Math.min(Math.floor(seismic.series.length / 3), 200));
          const ftleRes = await api.ftleField({
            series: seismic.series,
            window_size: winSize,
            dt: p.dt,
            dimension: p.dimension,
            tau: p.tau,
          });
          if (controller.signal.aborted) return;

          setFtle(ftleRes);
          const crit = ftleRes.field.filter((v) => v > 0.5).length;
          setCritCount(crit);

          const topEvents: AnomalyEvent[] = seismic.series
            .map((mag, i) => ({ mag, i }))
            .sort((a, b) => b.mag - a.mag)
            .slice(0, 4)
            .map(({ mag, i }, n) => ({
              id: `SEI-${i}`,
              title: `Seismic Event #${i}`,
              sub: `MAG ${mag.toFixed(1)} // USGS FEED`,
              severity: mag > 4 ? "critical" : mag > 2.5 ? "warning" : "nominal",
              time: n === 0 ? "NOW" : `${n * 4}M AGO`,
            }));
          setEvents(topEvents);
        }
      } catch (e) {
        if (!controller.signal.aborted) {
          console.error("[ArgusPage] load error:", e);
        }
      } finally {
        if (!controller.signal.aborted) setLoading(false);
      }
    }

    load();
    return () => controller.abort();
  }, []);

  return (
    <div className="min-h-screen pb-28">
      <PhantomHeader
        section="ARGUS"
        subtitle={loading ? "scanning…" : "live stream"}
        status={loading ? "idle" : "live"}
        icon="radar"
      />

      <main className="px-4 space-y-5">
        <div className="grid grid-cols-3 gap-3 fade-up fade-up-1">
          <MiniCard label="Seismic" value={seismicCount ? `${seismicCount}` : "—"} />
          <MiniCard label="Flights" value={flightCount ? flightCount.toLocaleString() : "—"} />
          <MiniCard label="Critical" value={`${critCount}`} color={critCount > 0 ? "text-alert" : "text-white/40"} />
        </div>

        <section className="glass-card rounded-[2rem] overflow-hidden fade-up fade-up-2">
          <div className="p-5 border-b" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
            <p className="label mb-1.5">Geospatial Anomaly Mesh</p>
            <div className="flex items-baseline gap-2">
              <span className="text-2xl font-light">
                {loading ? "Scanning…" : ftle ? "FTLE Field Active" : "Awaiting Data"}
              </span>
              {loading && <span className="w-2 h-2 rounded-full bg-alert animate-pulse" />}
            </div>
          </div>

          <div style={{ height: "80px", overflow: "hidden" }}>
            <svg viewBox="0 0 400 80" className="w-full opacity-25">
              <defs>
                <linearGradient id="meshGrad" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%"   stopColor="#ff007a" stopOpacity="0" />
                  <stop offset="40%"  stopColor="#ff007a" stopOpacity="0.8" />
                  <stop offset="70%"  stopColor="#ffbb00" stopOpacity="0.6" />
                  <stop offset="100%" stopColor="#00ff88" stopOpacity="0" />
                </linearGradient>
              </defs>
              {[0, 1, 2, 3, 4, 5].map((i) => (
                <line key={i} x1={i * 80} y1="0" x2={i * 60 + 30} y2="80"
                  stroke="url(#meshGrad)" strokeWidth="0.5" />
              ))}
              {[10, 30, 50, 70].map((y, i) => (
                <path key={i}
                  d={`M0,${y} Q100,${y + (i % 2 === 0 ? -15 : 15)} 200,${y} T400,${y}`}
                  fill="none" stroke="rgba(255,0,122,0.3)" strokeWidth="0.4" />
              ))}
              <circle cx="160" cy="35" r="3" fill="#ff007a" opacity="0.8" />
              <circle cx="280" cy="55" r="1.5" fill="#ffbb00" opacity="0.6" />
              <circle cx="90"  cy="20" r="2"   fill="#ff007a" opacity="0.4" />
            </svg>
          </div>

          <div style={{ height: "160px" }} className="px-3 pb-3">
            <FtleHeatmap ftle={ftle} loading={loading} bare />
          </div>
        </section>

        <section className="space-y-3 fade-up fade-up-3">
          <div className="flex items-center justify-between px-1">
            <p className="label">Anomaly Feed</p>
            <div className="flex items-center gap-2">
              <span className="w-1 h-1 rounded-full bg-alert animate-pulse" />
              <span className="font-mono text-[7px] text-alert tracking-widest">LIVE_STREAM</span>
            </div>
          </div>

          {loading ? (
            <div className="glass-card rounded-2xl p-5">
              <span className="label">fetching seismic feed…</span>
            </div>
          ) : events.length > 0 ? (
            events.map((ev) => <AnomalyCard key={ev.id} event={ev} />)
          ) : (
            <div className="glass-card rounded-2xl p-5 text-center">
              <span className="label">no anomalies detected</span>
            </div>
          )}
        </section>
      </main>

      <NavPill />
    </div>
  );
}

function AnomalyCard({ event }: { event: AnomalyEvent }) {
  const isCrit    = event.severity === "critical";
  const isWarning = event.severity === "warning";
  const color     = isCrit ? "#ff007a" : isWarning ? "#ffbb00" : "rgba(255,255,255,0.2)";
  const icon      = isCrit ? "crisis_alert" : isWarning ? "warning" : "analytics";

  return (
    <div className="glass-card rounded-2xl p-5 relative overflow-hidden"
      style={{ borderColor: isCrit ? "rgba(255,0,122,0.25)" : isWarning ? "rgba(255,187,0,0.2)" : undefined }}>
      {isCrit && (
        <div className="absolute inset-0 rounded-2xl"
          style={{ background: "radial-gradient(ellipse at left center, rgba(255,0,122,0.10) 0%, transparent 60%)" }} />
      )}
      <div className="relative flex items-center justify-between">
        <div className="flex items-center gap-4">
          <span className="material-symbols-outlined text-4xl"
            style={{ color: `${color}99`, fontVariationSettings: "'wght' 100, 'FILL' 0" }}>
            {icon}
          </span>
          <div className="flex flex-col gap-0.5">
            <span className="text-sm font-light tracking-wider uppercase" style={{ color }}>
              {event.title}
            </span>
            <span className="font-mono text-[9px] text-white/30 tracking-widest">{event.sub}</span>
          </div>
        </div>
        <span className="font-mono text-[9px] border px-2 py-0.5 rounded"
          style={{ color, borderColor: `${color}44` }}>
          {event.time}
        </span>
      </div>
    </div>
  );
}
