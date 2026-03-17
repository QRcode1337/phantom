"use client";

import dynamic from "next/dynamic";
import { ChartPanel, ChartEmpty } from "@/components/ChartPanel";
import type { EsnTrainResponse } from "@/lib/api";

const Plot = dynamic(() => import("react-plotly.js"), { ssr: false });

interface Props {
  esn?: EsnTrainResponse;
  loading?: boolean;
  bare?: boolean;
}

export default function EsnPrediction({ esn, loading, bare }: Props) {
  const xs = esn ? esn.actuals.map((_, i) => i) : [];
  const title = esn ? `ESN — MSE ${esn.mse.toExponential(2)}` : "ESN Prediction";

  const inner = !esn || esn.actuals.length === 0 ? (
    <ChartEmpty loading={loading} />
  ) : (
    <Plot
      data={[
        {
          x: xs, y: esn.actuals,
          type: "scatter", mode: "lines", name: "Actual",
          line: { color: "rgba(255,255,255,0.45)", width: 1.2 },
          hovertemplate: "t=%{x}<br>actual=%{y:.4f}<extra></extra>",
        },
        {
          x: xs, y: esn.predictions,
          type: "scatter", mode: "lines", name: "Predicted",
          line: { color: "#00ff88", width: 1.5, dash: "dot" },
          hovertemplate: "t=%{x}<br>pred=%{y:.4f}<extra></extra>",
        },
      ]}
      layout={{
        paper_bgcolor: "transparent", plot_bgcolor: "transparent",
        font: { color: "rgba(255,255,255,0.3)", size: 9, family: "'JetBrains Mono'" },
        margin: { t: 4, b: 32, l: 40, r: 8 },
        xaxis: { gridcolor: "rgba(255,255,255,0.04)", zeroline: false, tickfont: { size: 8 } },
        yaxis: { gridcolor: "rgba(255,255,255,0.04)", zeroline: false },
        legend: { x: 0.01, y: 0.99, bgcolor: "transparent", font: { size: 9, color: "rgba(255,255,255,0.4)" } },
      }}
      config={{ displayModeBar: false, responsive: true }}
      style={{ width: "100%", height: "100%" }}
      useResizeHandler
    />
  );

  if (bare) return <div className="w-full h-full">{inner}</div>;
  return <ChartPanel title={title} loading={loading}>{inner}</ChartPanel>;
}
