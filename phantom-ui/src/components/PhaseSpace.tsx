"use client";

import dynamic from "next/dynamic";
import { ChartPanel, ChartEmpty } from "@/components/ChartPanel";
import type { EmbedResponse } from "@/lib/api";

const Plot = dynamic(() => import("react-plotly.js"), { ssr: false });

interface Props {
  embed?: EmbedResponse;
  loading?: boolean;
  bare?: boolean;
}

const COMMON_LAYOUT = {
  paper_bgcolor: "transparent",
  font: { color: "rgba(255,255,255,0.3)", size: 9, family: "'JetBrains Mono'" },
  margin: { t: 0, b: 0, l: 0, r: 0 },
  showlegend: false,
} as const;

export default function PhaseSpace({ embed, loading, bare }: Props) {
  const vectors = embed?.vectors ?? [];
  const n       = vectors.length;
  const is3d    = n > 0 && vectors[0].length >= 3;
  const colors  = vectors.map((_, i) => i / Math.max(n - 1, 1));

  const inner = n === 0 ? (
    <ChartEmpty loading={loading} />
  ) : is3d ? (
    <Plot
      data={[{
        type: "scatter3d",
        x: vectors.map((v) => v[0]),
        y: vectors.map((v) => v[1]),
        z: vectors.map((v) => v[2]),
        mode: "lines+markers",
        marker: {
          size: 2, color: colors,
          colorscale: [[0, "#00ff88"], [0.5, "#0066ff"], [1, "#ff007a"]],
          showscale: false,
        },
        line: { color: "#00ff88", width: 0.8 },
        hoverinfo: "skip",
      }]}
      layout={{
        ...COMMON_LAYOUT,
        scene: {
          bgcolor: "transparent",
          xaxis: { gridcolor: "rgba(255,255,255,0.04)", color: "rgba(255,255,255,0.2)", zeroline: false },
          yaxis: { gridcolor: "rgba(255,255,255,0.04)", color: "rgba(255,255,255,0.2)", zeroline: false },
          zaxis: { gridcolor: "rgba(255,255,255,0.04)", color: "rgba(255,255,255,0.2)", zeroline: false },
        },
      }}
      config={{ displayModeBar: false, responsive: true }}
      style={{ width: "100%", height: "100%" }}
      useResizeHandler
    />
  ) : (
    <Plot
      data={[{
        type: "scatter",
        x: vectors.map((v) => v[0]),
        y: vectors.map((v) => v[1]),
        mode: "lines+markers",
        marker: { size: 2.5, color: colors, colorscale: [[0, "#00ff88"], [1, "#ff007a"]] },
        line: { color: "#00ff88", width: 1 },
        hoverinfo: "skip",
      }]}
      layout={{
        ...COMMON_LAYOUT,
        plot_bgcolor: "transparent",
        margin: { t: 4, b: 28, l: 36, r: 8 },
        xaxis: { gridcolor: "rgba(255,255,255,0.04)", zeroline: false },
        yaxis: { gridcolor: "rgba(255,255,255,0.04)", zeroline: false },
      }}
      config={{ displayModeBar: false, responsive: true }}
      style={{ width: "100%", height: "100%" }}
      useResizeHandler
    />
  );

  if (bare) return <div className="w-full h-full">{inner}</div>;
  return <ChartPanel title="Phase Space Embedding" loading={loading}>{inner}</ChartPanel>;
}
