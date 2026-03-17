"use client";

import dynamic from "next/dynamic";
import { ChartPanel, ChartEmpty } from "@/components/ChartPanel";
import type { FtleFieldResponse } from "@/lib/api";

const Plot = dynamic(() => import("react-plotly.js"), { ssr: false });

interface Props {
  ftle?: FtleFieldResponse;
  loading?: boolean;
  bare?: boolean;
}

export default function FtleHeatmap({ ftle, loading, bare }: Props) {
  const hasData = ftle && ftle.field.length > 0;

  const inner = !hasData ? (
    <ChartEmpty loading={loading} />
  ) : (
    <Plot
      data={[{
        type: "heatmap",
        z: [ftle.field],
        x: ftle.positions,
        colorscale: [[0, "#070708"], [0.4, "#00ff88"], [0.7, "#ffbb00"], [1, "#ff007a"]],
        showscale: true,
        colorbar: {
          thickness: 8,
          tickfont: { color: "rgba(255,255,255,0.3)", size: 8, family: "'JetBrains Mono'" },
          len: 0.85,
          x: 1.01,
        },
        hovertemplate: "pos: %{x}<br>FTLE: %{z:.4f}<extra></extra>",
      }]}
      layout={{
        paper_bgcolor: "transparent", plot_bgcolor: "transparent",
        font: { color: "rgba(255,255,255,0.3)", size: 9, family: "'JetBrains Mono'" },
        margin: { t: 4, b: 32, l: 36, r: 56 },
        xaxis: { gridcolor: "rgba(255,255,255,0.04)", zeroline: false, tickfont: { size: 8 } },
        yaxis: { visible: false },
      }}
      config={{ displayModeBar: false, responsive: true }}
      style={{ width: "100%", height: "100%" }}
      useResizeHandler
    />
  );

  if (bare) return <div className="w-full h-full">{inner}</div>;
  return <ChartPanel title="FTLE Field" loading={loading} height={200}>{inner}</ChartPanel>;
}
