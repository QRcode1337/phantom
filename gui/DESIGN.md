# Phantom Design System

## 1. Brand Identity
**Name:** Phantom  
**Vibe:** Tactical OS, Deep Space Glassmorphism, highly advanced abstract interface.  
**Core Purpose:** Visualizing chaos mathematics and regime shifts through glowing neon accents floating over an infinite dark abyss.  

## 2. Color Palette
- **Background:** Radial gradient from Deep Purple (`#2d1b4d`) to Absolute Charcoal (`#0a0a0c`).
- **Surface/Card:** Frost Glass – a semi-transparent white wash (`bg-white/5` or `rgba(255,255,255,0.05)`) with an extreme backdrop blur (`blur-2xl`) and delicate, hair-thin white borders (`border-white/10`).
- **Dividers:** Ultra-thin, semi-transparent white lines (`bg-white/5`).
- **Primary Text:** Pure White or high opacity white (`text-white/90`).
- **Secondary Text:** Dimmed white (`text-white/40` or `text-white/20`).

### Signal Colors
- **Enter/Stable/Primary System:** `#00ff88` (Aurora Green). Often used for subtle glowing shadows (`shadow-[0_0_8px_#00ff88]`) or radiant background gradients.
- **Watch/Transitioning:** `#ffbb00` (Solar Yellow).
- **Skip/Chaotic:** `#1a1a1a` (Dark Slate).
- **Critical/Anomaly/Radar:** `#ff3333` (Alert Red) and/or `#ff00ff` (Hot Pink/Magenta for abstract nodes).

## 3. Typography
- **Headings & UI:** `Inter` sans-serif, specifically utilizing ultra-light weights ("hairline" `wght: 100`). Extremely wide tracking/letter-spacing for section headers (`tracking-[0.4em]`).
- **Data & Numbers:** `JetBrains Mono` monospace, also heavily utilizing ultra-light weights.
- **Mix:** The typography should feel ghostly, floating, and precise.

## 4. Component Styles
- **Cards/Panels:** Large border radii on main cards (`rounded-3xl` or `rounded-[2.5rem]`). Deep inset shadows or drop shadows.
- **Badges/Tags:** Barely-there outlines or simple glowing dots of colored light.
- **Abstract Data (CRITICAL):** MUST represent data via highly complex 3D vector visuals. Use intricate glowing SVG graphs (e.g. isometric views of chaotic attractors, complex web-like meshes, abstract 3D ribbons twisting in space, interconnected node constellations). Make use of transparency, drop shadows, gradient strokes, and multi-layered overlapping paths to create profound depth and complexity.
- **Layout:** Floating panels over a fixed background.

## 6. Design System Notes for Stitch Generation
**(Copy this block precisely into next-prompt.md for Stitch)**

*   **Theme:** "Tactical Glass OS". Deep space gradient background (dark purple to absolute charcoal). UI elements are heavily blurred glass panels (`bg-white/5`, `backdrop-blur-2xl`) with ultra-thin white borders and large rounded corners (`2.5rem`).
*   **Typography:** Floating, precise typography. Utilize ultra-light "hairline" weights (`100`) for both `Inter` (sans-serif UI) and `JetBrains Mono` (Data/Numbers). Use extremely wide letter-spacing (`tracking-widest`) for small uppercase labels.
*   **Color Palette:** Pure white text (often semi-transparent at 40% or 60%). Neon glowing accents: Aurora Green (`#00ff88`) for system status/positive edge, Solar Yellow (`#ffbb00`) for warnings, Alert Red (`#ff3333`) or Hot Pink (`#ff00ff`) for critical anomalies.
*   **Vibe & Visuals (CRITICAL):** Highly advanced, ambient, abstract military OS. Centerpiece data visualizations MUST be extremely complex abstract 3D vector drawings/SVGs. Imagine an isometric view of a Strange Attractor, overlapping twisted 3D ribbons with gradient strokes, or deep layered node constellation meshes. Use transparency, drop shadows, and overlapping vector elements to create deep 3D complexity.
