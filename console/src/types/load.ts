export type LoadStatus = "loading" | "failed" | "missing" | "ready";

export interface LoadState {
  status: LoadStatus;
  error: string | null;
  retry: () => void;
}
