interface Props {
  title: string;
  loading?: boolean;
  height?: number;
  children: React.ReactNode;
}

export function ChartPanel({ title, loading, height = 280, children }: Props) {
  return (
    <div className="glass-card rounded-3xl overflow-hidden flex flex-col" style={{ height }}>
      <div
        className="flex items-center justify-between px-4 py-3 border-b shrink-0"
        style={{ borderColor: "rgba(255,255,255,0.06)" }}
      >
        <span className="label">{title}</span>
        {loading && (
          <span className="font-mono text-[8px] text-primary animate-pulse tracking-widest">
            COMPUTING
          </span>
        )}
      </div>
      <div className="flex-1 p-2 min-h-0">{children}</div>
    </div>
  );
}

export function ChartEmpty({ loading, label }: { loading?: boolean; label?: string }) {
  return (
    <div className="h-full flex items-center justify-center">
      <span className="label">
        {loading ? "computing…" : label ?? "load data to begin"}
      </span>
    </div>
  );
}
