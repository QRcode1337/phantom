"use client";

interface Props {
  section: string;
  subtitle?: string;
  status?: "live" | "idle" | "error";
  icon?: string;
}

export default function PhantomHeader({ section, subtitle, status = "idle", icon }: Props) {
  const dotColor =
    status === "live"  ? "bg-primary glow-green" :
    status === "error" ? "bg-alert glow-red"      :
    "bg-white/20";

  return (
    <header
      className="sticky top-0 z-40 px-5 pt-10 pb-5 flex items-center justify-between"
      style={{ backdropFilter: "blur(16px)", WebkitBackdropFilter: "blur(16px)" }}
    >
      <div className="flex items-center gap-3.5">
        {icon && (
          <div className="glass-card w-10 h-10 rounded-2xl flex items-center justify-center shrink-0 relative">
            <span
              className="material-symbols-outlined text-primary text-lg"
              style={{ fontVariationSettings: "'wght' 200, 'FILL' 0", filter: "drop-shadow(0 0 4px rgba(0,255,136,0.4))" }}
            >
              {icon}
            </span>
          </div>
        )}
        <div className="flex flex-col gap-0.5">
          <p className="font-mono text-[8px] tracking-[0.45em] text-white/25 uppercase">PHANTOM //</p>
          <h1 className="text-lg font-semibold tracking-wide leading-none" style={{ fontFamily: "'Syne', sans-serif" }}>
            {section}
          </h1>
        </div>
      </div>

      {subtitle && (
        <div className="flex items-center gap-2">
          <span className={`w-1.5 h-1.5 rounded-full ${dotColor} ${status === "live" ? "animate-pulse" : ""}`} />
          <span className="label">{subtitle}</span>
        </div>
      )}
    </header>
  );
}
