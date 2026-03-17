# Phantom Analysis Workbench — Design System

## 1. Aesthetic
- **Core Concept:** "Analytical Chaos" — Dark, technical, precise, yet fluid.
- **Colors:** Deep obsidian backgrounds (#0B0E14), neon-amber accents (#F59E0B), and slate-gray secondary elements.
- **Visuals:** Use glowing gradients for regime boundaries and subtle grid patterns.

## 2. Typography
- **Monospace:** `Fira Code` or `JetBrains Mono` for data and parameters.
- **Sans-Serif:** `Inter` for UI elements and headers.

## 3. UI Components
- **Chart Panels:** Frameless cards with neon top-borders, background color slightly lighter than the page.
- **Sliders:** Modern, slim sliders with amber "fill" and numeric readout.
- **Status Dots:** Glowing LEDs: Green (Normal), Amber (Transitioning/Rate Limited), Red (Critical Chaos/Error).

## 4. Visualization Rules
- **Stable (<0.3):** Subtle green glow.
- **Transitioning (0.3-0.7):** Pulsing amber.
- **Chaotic (>0.7):** Jagged red/orange lines.
- **Phase Space:** 3D scatter with a "comet trail" (points fade from bright to dark as they age).

## 5. Design System Notes for Stitch Generation
- Use a **dark theme** as the primary background.
- Charts should use **obsidian backgrounds** with white/gray grid lines.
- Accent color for primary actions: **Amber (#F59E0B)**.
- Sidebar should be a **semi-transparent glassmorphism** effect over the background.
- Include **monospaced data readouts** for values like $\lambda$, MSE, and $m$.
- Layout must be a **2x2 grid** for charts with a **Left Sidebar** for controls.
