"use client";

import { useState, useCallback, useRef } from "react";
import NavPill from "@/components/NavPill";
import DataLoader from "@/components/DataLoader";
import ParameterTuner, { Params, DEFAULT_PARAMS } from "@/components/ParameterTuner";
import ChaosTimeline from "@/components/ChaosTimeline";
import PhaseSpace from "@/components/PhaseSpace";
import FtleHeatmap from "@/components/FtleHeatmap";
import EsnPrediction from "@/components/EsnPrediction";
import { api, AnalyzeResponse, EmbedResponse, FtleFieldResponse, EsnTrainResponse } from "@/lib/api";

interface State {
  analyzeResults: AnalyzeResponse[];
  embed?: EmbedResponse;
  ftle?: FtleFieldResponse;
  esn?: EsnTrainResponse;
  error?: string;
}

export default function LabPage() {
  const [series, setSeries] = useState<number[]>([]);
  const [params, setParams] = useState<Params>(DEFAULT_PARAMS);
  const [loading, setLoading] = useState(false);
  const [state, setState] = useState<State>({ analyzeResults: [] });
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const runAnalysis = useCallback(async (s: number[], p: Params) => {
    const minRequired = p.dimension * p.tau + 1;
    if (s.length < minRequired) {
      setState({ analyzeResults: [], error: `Need at least ${minRequired} points for dim=${p.dimension}, tau=${p.tau}. Have ${s.length}.` });
      return;
    }
    setLoading(true);
    setState((prev) => ({ ...prev, error: undefined }));
    try {
      const windowSize = Math.min(Math.max(p.window_size, 30), Math.floor(s.length / 2));
      const step = Math.max(1, Math.ceil((s.length - windowSize) / 50));
      const windows: number[][] = [];
      for (let i = 0; i + windowSize <= s.length; i += step) windows.push(s.slice(i, i + windowSize));

      const [analyzeResults, embed, ftle, esn] = await Promise.all([
        Promise.all(windows.map((w) => api.analyze({ series: w, dt: p.dt, dimension: p.dimension, tau: p.tau }))),
        api.embed({ series: s, dimension: p.dimension, tau: p.tau }),
        api.ftleField({ series: s, window_size: windowSize, dt: p.dt, dimension: p.dimension, tau: p.tau }),
        api.esnTrain({ series: s, reservoir_size: p.reservoir_size, spectral_radius: p.spectral_radius,
          leak_rate: p.leak_rate, connectivity: p.connectivity, input_scaling: p.input_scaling, ridge_param: p.ridge_param }),
      ]);
      setState({ analyzeResults, embed, ftle, esn });
    } catch (e) {
      setState((prev) => ({ ...prev, error: e instanceof Error ? e.message : "Analysis failed" }));
    } finally {
      setLoading(false);
    }
  }, []);

  function trigger(s: number[], p: Params) {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => runAnalysis(s, p), 500);
  }

  const regime = state.analyzeResults.at(-1)?.regime;
  const score  = state.analyzeResults.at(-1)?.chaos_score;

  const regimeColor = regime === "chaotic" ? "text-alert" : regime === "transitioning" ? "text-solar" : "text-primary";
  const regimeBorder = regime === "chaotic" ? "rgba(255,0,122,0.4)" : regime === "transitioning" ? "rgba(255,187,0,0.4)" : "rgba(0,255,136,0.4)";

  return (
    <div className="min-h-screen flex flex-col pb-24">
      {/* Header */}
      <header className="sticky top-0 z-40 px-6 py-4 flex items-center gap-4"
        style={{ backdropFilter: "blur(16px)", WebkitBackdropFilter: "blur(16px)",
          borderBottom: "0.5px solid rgba(255,255,255,0.07)" }}>
        <div className="flex flex-col">
          <p className="font-mono text-[8px] tracking-[0.45em] text-white/25 uppercase">PHANTOM //</p>
          <h1 className="text-base font-semibold tracking-wide">ANALYSIS LAB</h1>
        </div>

        {score !== undefined && (
          <div className="flex items-center gap-3 ml-4">
            <span
              className={`font-mono text-[9px] border px-2 py-0.5 rounded ${regimeColor}`}
              style={{ borderColor: regimeBorder }}
            >
              {regime?.toUpperCase()}
            </span>
            <span className="font-mono text-xs text-white/30">
              chaos <span className="text-white/70">{score.toFixed(3)}</span>
            </span>
          </div>
        )}

        {state.error && <span className="ml-auto font-mono text-[9px] text-alert">{state.error}</span>}
      </header>

      {/* Layout */}
      <div className="flex flex-1 gap-4 p-4 overflow-auto min-h-0">
        {/* Sidebar */}
        <aside className="w-64 shrink-0 flex flex-col gap-4">
          <DataLoader onData={(s) => { setSeries(s); if (s.length >= 22) trigger(s, params); }} />
          <ParameterTuner onApply={(p) => { setParams(p); if (series.length >= p.dimension * p.tau + 1) trigger(series, p); }} disabled={loading} seriesLength={series.length} />
        </aside>

        {/* Chart grid */}
        <main className="flex-1 grid grid-cols-2 gap-4 content-start min-w-0">
          <ChaosTimeline results={state.analyzeResults} loading={loading} />
          <PhaseSpace embed={state.embed} loading={loading} />
          <div className="col-span-2">
            <FtleHeatmap ftle={state.ftle} loading={loading} />
          </div>
          <div className="col-span-2">
            <EsnPrediction esn={state.esn} loading={loading} />
          </div>
        </main>
      </div>

      <NavPill />
    </div>
  );
}
