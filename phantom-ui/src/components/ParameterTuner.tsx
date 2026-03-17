"use client";

import { useState } from "react";

export interface Params {
  dimension: number;
  tau: number;
  window_size: number;
  dt: number;
  reservoir_size: number;
  spectral_radius: number;
  leak_rate: number;
  connectivity: number;
  input_scaling: number;
  ridge_param: number;
}

export const DEFAULT_PARAMS: Params = {
  dimension: 3, tau: 1, window_size: 50, dt: 1.0,
  reservoir_size: 100, spectral_radius: 0.9, leak_rate: 0.3,
  connectivity: 0.1, input_scaling: 1.0, ridge_param: 1e-6,
};

interface SliderDef {
  key: keyof Params;
  label: string;
  min: number;
  max: number;
  step: number;
  fmt?: (v: number) => string;
}

const SLIDERS: SliderDef[] = [
  { key: "dimension",      label: "Embed Dim",   min: 2,   max: 10,   step: 1 },
  { key: "tau",            label: "Embed Tau",   min: 1,   max: 20,   step: 1 },
  { key: "window_size",    label: "FTLE Window", min: 10,  max: 200,  step: 5 },
  { key: "dt",             label: "dt",          min: 0.1, max: 10,   step: 0.1, fmt: (v) => v.toFixed(1) },
  { key: "reservoir_size", label: "Reservoir",   min: 20,  max: 500,  step: 10 },
  { key: "spectral_radius",label: "Spectral ρ",  min: 0.1, max: 0.99, step: 0.01, fmt: (v) => v.toFixed(2) },
  { key: "leak_rate",      label: "Leak Rate",   min: 0.01,max: 1.0,  step: 0.01, fmt: (v) => v.toFixed(2) },
  { key: "connectivity",   label: "Connectivity",min: 0.01,max: 1.0,  step: 0.01, fmt: (v) => v.toFixed(2) },
  { key: "input_scaling",  label: "Input Scale", min: 0.1, max: 5.0,  step: 0.1,  fmt: (v) => v.toFixed(1) },
];

interface Props { onApply: (p: Params) => void; disabled?: boolean; seriesLength?: number; }

export default function ParameterTuner({ onApply, disabled, seriesLength }: Props) {
  const [p, setP] = useState<Params>(DEFAULT_PARAMS);
  const [ridgeLog, setRidgeLog] = useState(-6);

  const minPoints = p.dimension * p.tau + 1;
  const paramsValid = !seriesLength || seriesLength >= minPoints;

  return (
    <div className="glass-card rounded-3xl p-4 flex flex-col gap-3">
      <p className="label">Parameters</p>

      <div className="flex flex-col gap-2.5">
        {SLIDERS.map((s) => (
          <div key={s.key} className="flex items-center gap-2">
            <label className="w-24 font-mono text-[8px] tracking-widest text-white/30 uppercase shrink-0">{s.label}</label>
            <input
              type="range" min={s.min} max={s.max} step={s.step}
              value={p[s.key] as number} disabled={disabled}
              onChange={(e) => setP((prev) => ({ ...prev, [s.key]: Number(e.target.value) }))}
              className="flex-1"
              style={{ accentColor: "#00ff88" }}
            />
            <span className="w-12 font-mono text-[9px] text-white/50 text-right">
              {s.fmt ? s.fmt(p[s.key] as number) : p[s.key]}
            </span>
          </div>
        ))}

        {/* Ridge param — log scale */}
        <div className="flex items-center gap-2">
          <label className="w-24 font-mono text-[8px] tracking-widest text-white/30 uppercase shrink-0">Ridge λ</label>
          <input type="range" min={-10} max={-1} step={0.5} value={ridgeLog} disabled={disabled}
            onChange={(e) => setRidgeLog(Number(e.target.value))}
            className="flex-1" style={{ accentColor: "#00ff88" }} />
          <span className="w-12 font-mono text-[9px] text-white/50 text-right">1e{ridgeLog}</span>
        </div>
      </div>

      {seriesLength !== undefined && !paramsValid && (
        <p className="font-mono text-[8px] text-alert">
          Need {minPoints} points, have {seriesLength}. Reduce dim or tau.
        </p>
      )}

      <button
        onClick={() => onApply({ ...p, ridge_param: Math.pow(10, ridgeLog) })}
        disabled={disabled || !paramsValid}
        className="py-2 rounded-full font-mono text-[9px] tracking-widest transition-all"
        style={{
          background: (disabled || !paramsValid) ? "rgba(255,255,255,0.04)" : "rgba(0,255,136,0.10)",
          border: "0.5px solid rgba(0,255,136,0.25)",
          color: (disabled || !paramsValid) ? "rgba(255,255,255,0.2)" : "rgba(0,255,136,0.9)",
          cursor: (disabled || !paramsValid) ? "not-allowed" : "pointer",
        }}
      >
        {disabled ? "COMPUTING…" : !paramsValid ? "INSUFFICIENT DATA" : "APPLY"}
      </button>
    </div>
  );
}
