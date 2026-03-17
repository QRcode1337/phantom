interface Props {
  label: string;
  value: string;
  color?: string;
}

export default function MiniCard({ label, value, color = "text-white/90" }: Props) {
  return (
    <div className="glass-card rounded-2xl p-4 flex flex-col gap-1">
      <span className="label">{label}</span>
      <span className={`font-mono text-xl font-light ${color}`}>{value}</span>
    </div>
  );
}
