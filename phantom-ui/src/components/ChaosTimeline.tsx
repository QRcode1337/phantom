"use client";

import dynamic from "next/dynamic";
import { ChartPanel, ChartEmpty } from "@/components/ChartPanel";
import type { AnalyzeResponse } from "@/lib/api";

const Plot = dynamic(() => import("react-plotly.js"), { ssr: false });

interface Props {
  results: AnalyzeResponse[];
  loading?: boolean;
  bare?: boolean;
}

const BANDS = [
  { y0: 0,   y1: 0.3,  color: "rgba(0,255,136,0.07)"  },
  { y0: 0.3, y1: 0.7,  color: "rgba(255,187,0,0.07)"  },
  { y0: 0.7, y1: 1.0,  color: "rgba(255,0,122,0.10)"  },
];

export default function ChaosTimeline({ results, loading, bare }: Props) {
  const xs      = results.map((_, i) => i);
  const scores  = results.map((r) => r.chaos_score);
  const regimes = results.map((r) => r.regime);

  const shapes = BANDS.map(({ y0, y1, color }) => ({
    type: "rect" as const, xref: "paper" as const, yref: "y" as const,
    x0: 0, x1: 1, y0, y1, fillcolor: color, line: { width: 0 },
  }));

  const inner = results.length === 0 ? (
    <ChartEmpty loading={loading} />
  ) : (
    <Plot
      data={[{
        x: xs, y: scores,
        type: "scatter", mode: "lines+markers",
        marker: {
          color: regimes.map((r) =>
            r === "chaotic" ? "#ff007a" : r === "transitioning" ? "#ffbb00" : "#00ff88"
          ),
          size: 3,
        },
        line: { color: "#00ff88", width: 1.5 },
        name: "Chaos Score",
        hovertemplate: "window %{x}<br>score: %{y:.3f}<extra></extra>",
      }]}
      layout={{
        shapes,
        paper_bgcolor: "transparent", plot_bgcolor: "transparent",
        font: { color: "rgba(255,255,255,0.35)", size: 10, family: "'JetBrains Mono'" },
        margin: { t: 6, b: 32, l: 40, r: 8 },
        yaxis: { range: [0, 1], gridcolor: "rgba(255,255,255,0.04)", zeroline: false, tickfont: { size: 9 } },
        xaxis: { gridcolor: "rgba(255,255,255,0.04)", zeroline: false, tickfont: { size: 9 } },
        showlegend: false,
      }}
      config={{ displayModeBar: false, responsive: true }}
      style={{ width: "100%", height: "100%" }}
      useResizeHandler
    />
  );

  if (bare) return <div className="w-full h-full">{inner}</div>;
  return <ChartPanel title="Chaos Score Timeline" loading={loading}>{inner}</ChartPanel>;
}
