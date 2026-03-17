const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

// ─── Request / Response types ───────────────────────────────────────────────

export interface AnalyzeRequest {
  series: number[];
  dt: number;
  dimension?: number;
  tau?: number;
}

export interface AnalyzeResponse {
  chaos_score: number;
  lambda: number;
  lyapunov_time: number;
  doubling_time: number;
  regime: "stable" | "transitioning" | "chaotic";
  points_used: number;
  dimension: number;
  pairs_found: number;
}

export interface EmbedRequest {
  series: number[];
  dimension?: number;
  tau?: number;
}

export interface EmbedResponse {
  vectors: number[][];
  dimension: number;
  tau: number;
  num_vectors: number;
}

export interface FtleFieldRequest {
  series: number[];
  window_size: number;
  dt: number;
  dimension?: number;
  tau?: number;
}

export interface FtleFieldResponse {
  field: number[];
  positions: number[];
  window_size: number;
  series_len: number;
  embedded_len: number;
}

export interface EsnTrainRequest {
  series: number[];
  reservoir_size?: number;
  spectral_radius?: number;
  leak_rate?: number;
  ridge_param?: number;
  connectivity?: number;
  input_scaling?: number;
  seed?: number;
}

export interface EsnTrainResponse {
  predictions: number[];
  actuals: number[];
  mse: number;
  training_samples: number;
  reservoir_size: number;
  dimension: number;
}

export interface FeedResponse {
  series: number[];
  source: string;
  points: number;
}

export interface HealthResponse {
  status: string;
  feeds: Record<string, string>;
}

// ─── Signal persistence types ─────────────────────────────────────────────────

export interface SignalRecord {
  timestamp: string; // ISO-8601 UTC
  signal_type: string;
  action: "ENTER" | "WATCH" | "SKIP";
  edge: number;
  chaos_score: number;
  market_type: string;
  direction: "YES" | "NO";
  confidence: "HIGH" | "MEDIUM" | "LOW";
  reason: string;
}

export interface RecordSignalRequest {
  signal_type: string;
  action: string;
  edge: number;
  chaos_score: number;
  market_type: string;
  direction: string;
  confidence: string;
  reason: string;
  timestamp?: string; // ISO-8601 override; defaults to server time
}

export interface RecordSignalResponse {
  stored: boolean;
  total_count: number;
  record: SignalRecord;
}

export interface SignalsResponse {
  records: SignalRecord[];
  count: number;
  total_stored: number;
}

export interface PersistableSignal {
  record: SignalRecord;
  persisted: boolean;
}

export interface AnalyzeSignalsRequest {
  lat?: number;
  lon?: number;
  weather_hours?: number;
  weather_target?: number;
  weather_above?: boolean;
  btc_days?: number;
}

export interface AnalyzeSignalsResponse {
  signals: PersistableSignal[];
  persisted_count: number;
  errors: string[];
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const json = await res.json();
  if (!res.ok) throw new Error(json.error ?? `HTTP ${res.status}`);
  return json as T;
}

async function get<T>(path: string, params?: Record<string, string | number | undefined>): Promise<T> {
  const url = new URL(`${API_BASE}${path}`);
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      if (v !== undefined) url.searchParams.set(k, String(v));
    }
  }
  const res = await fetch(url.toString());
  const json = await res.json();
  if (!res.ok) throw new Error(json.error ?? `HTTP ${res.status}`);
  return json as T;
}

// ─── API calls ───────────────────────────────────────────────────────────────

export const api = {
  analyze: (req: AnalyzeRequest) => post<AnalyzeResponse>("/api/analyze", req),
  embed: (req: EmbedRequest) => post<EmbedResponse>("/api/embed", req),
  ftleField: (req: FtleFieldRequest) => post<FtleFieldResponse>("/api/ftle-field", req),
  esnTrain: (req: EsnTrainRequest) => post<EsnTrainResponse>("/api/esn-train", req),

  feeds: {
    weather: (lat: number, lon: number, hours?: number) =>
      get<FeedResponse>("/api/feeds/weather", { lat, lon, hours }),
    seismic: (period: string) =>
      get<FeedResponse>("/api/feeds/seismic", { period }),
    btc: (days?: number) =>
      get<FeedResponse>("/api/feeds/btc", { days }),
    opensky: () =>
      get<{ states: unknown[]; count: number }>("/api/feeds/opensky"),
    health: () =>
      get<HealthResponse>("/api/health"),
  },

  signals: {
    /** Manually record a signal to the persistence store. */
    record: (req: RecordSignalRequest) =>
      post<RecordSignalResponse>("/api/signals/record", req),

    /**
     * Query stored signals.
     * @param since  ISO-8601 UTC lower bound (inclusive).
     * @param action Filter to "ENTER" | "WATCH" | "SKIP".
     * @param limit  Max records returned (default 200, most-recent first).
     */
    query: (params?: { since?: string; action?: string; limit?: number }) =>
      get<SignalsResponse>("/api/signals", params as Record<string, string | number | undefined>),

    /**
     * Trigger the full KalshiSignalEngine pipeline against live feeds,
     * persist actionable signals, and return all generated signals.
     */
    analyze: (req?: AnalyzeSignalsRequest) =>
      post<AnalyzeSignalsResponse>("/api/signals/analyze", req ?? {}),
  },
};
