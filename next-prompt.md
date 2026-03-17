---
page: index
---
The main dashboard for the Phantom Analysis Workbench. This iteration focuses on the overall layout and the **Chaos Score Timeline** (View 1).

**DESIGN SYSTEM (REQUIRED):**
- Use a **dark theme** (obsidian #0B0E14) as the primary background.
- Charts should use **obsidian backgrounds** with white/gray grid lines.
- Accent color for primary actions: **Amber (#F59E0B)**.
- Sidebar should be a **semi-transparent glassmorphism** effect over the background.
- Include **monospaced data readouts** for values like λ, MSE, and m.
- Layout must be a **2x2 grid** for charts with a **Left Sidebar** for controls.

**Page Structure:**
1. **Top Bar:** "Phantom Analysis Workbench" title, feed status indicators (4 LED dots: Weather, Seismic, BTC, Flights), and a "Data Loader" button.
2. **Left Sidebar (Parameter Tuner):** Sliders for Embedding (Dim, Tau), FTLE (k_fit, theiler), ESN (res_size, spectral), and a large amber "APPLY" button.
3. **Main Content (2x2 Grid):**
   - **Top Left (Chaos Timeline):** A Plotly line chart showing Chaos Score (0.0-1.0) over time with green/yellow/red horizontal bands.
   - **Top Right (Phase Space):** Placeholder for 3D scatter plot.
   - **Bottom Left (FTLE Heatmap):** Placeholder for heatmap.
   - **Bottom Right (ESN Prediction):** Placeholder for prediction overlay.
4. **Footer:** Small copyright and "Engine: Phantom v0.1.0".
