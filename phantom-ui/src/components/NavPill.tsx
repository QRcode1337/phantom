"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const LINKS = [
  { href: "/",        icon: "grid_view",  label: "Core"    },
  { href: "/weather", icon: "cloud",      label: "Weather" },
  { href: "/price",   icon: "show_chart", label: "Price"   },
  { href: "/argus",   icon: "radar",      label: "Argus"   },
  { href: "/signals", icon: "monitoring", label: "Signals" },
  { href: "/lab",     icon: "science",    label: "Lab"     },
];

export default function NavPill() {
  const path = usePathname();

  return (
    <nav className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50" style={{ width: "min(92%, 380px)" }}>
      <div className="glass-card-heavy rounded-full px-5 py-3.5 flex items-center justify-around">
        {LINKS.map(({ href, icon, label }) => {
          const active = path === href;
          return (
            <Link key={href} href={href} className="flex flex-col items-center gap-1 group relative">
              {/* Active indicator dot */}
              {active && (
                <span
                  className="absolute -top-2 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-primary pulse-glow"
                  style={{ boxShadow: "0 0 6px rgba(0,255,136,0.8)" }}
                />
              )}
              <span
                className={`material-symbols-outlined text-xl transition-all duration-300 ${
                  active
                    ? "text-primary"
                    : "text-white/25 group-hover:text-white/55"
                }`}
                style={{
                  fontVariationSettings: active ? "'wght' 300, 'FILL' 1" : "'wght' 200, 'FILL' 0",
                  filter: active ? "drop-shadow(0 0 6px rgba(0,255,136,0.5))" : "none",
                }}
              >
                {icon}
              </span>
              <span
                className={`font-mono text-[7px] tracking-[0.3em] uppercase transition-all duration-300 ${
                  active ? "text-primary/80" : "text-white/18 group-hover:text-white/40"
                }`}
              >
                {label}
              </span>
            </Link>
          );
        })}
      </div>
    </nav>
  );
}
