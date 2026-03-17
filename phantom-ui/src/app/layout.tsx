import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Phantom // Engine",
  description: "Chaos math anomaly detection — FTLE, ESN, Delay Embedding",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <head>
        {/* Syne — distinctive geometric display font */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Syne:wght@400;500;600;700;800&family=JetBrains+Mono:wght@100;200;300;400&display=swap"
          rel="stylesheet"
        />
        <link
          href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200"
          rel="stylesheet"
        />
      </head>
      <body className="min-h-screen overflow-x-hidden">
        {/* Ambient background orbs — positioned precisely */}
        <div aria-hidden className="fixed pointer-events-none"
          style={{ top: "-8%", right: "-4%", width: "420px", height: "420px",
            background: "radial-gradient(circle, rgba(0,255,136,0.055) 0%, transparent 70%)",
            filter: "blur(60px)" }} />
        <div aria-hidden className="fixed pointer-events-none"
          style={{ bottom: "10%", left: "-8%", width: "480px", height: "480px",
            background: "radial-gradient(circle, rgba(80,40,180,0.07) 0%, transparent 70%)",
            filter: "blur(80px)" }} />
        <div aria-hidden className="fixed pointer-events-none"
          style={{ top: "35%", right: "18%", width: "280px", height: "280px",
            background: "radial-gradient(circle, rgba(255,0,122,0.04) 0%, transparent 70%)",
            filter: "blur(60px)" }} />

        {children}
      </body>
    </html>
  );
}
