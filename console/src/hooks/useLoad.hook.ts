import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";
import { ApiError } from "../services/http";
import type { LoadState, LoadStatus } from "../types/load";
import { errorMessage, isUnauthorized, reportApiError } from "../utils/reportApiError";

export interface Loaded<T> extends LoadState {
  value: T | null;
  set: (value: T) => void;
}

interface LoadSignals<T> {
  value: Signal<T | null>;
  status: Signal<LoadStatus>;
  error: Signal<string | null>;
}

const settle = <T>(signals: LoadSignals<T>, failure: unknown): void => {
  if (failure instanceof ApiError && failure.status === 404) {
    signals.status.value = "missing";
    return;
  }
  if (isUnauthorized(failure)) {
    reportApiError(failure);
  }
  signals.status.value = "failed";
  signals.error.value = errorMessage(failure);
};

/** Starts one load and returns a function that ignores its late answer. */
const start = <T>(run: () => Promise<T>, signals: LoadSignals<T>): (() => void) => {
  const { value, status, error } = signals;
  let current = true;
  status.value = value.value === null ? "loading" : status.value;
  run()
    .then((result): void => {
      if (current) {
        value.value = result;
        status.value = "ready";
        error.value = null;
      }
    })
    .catch((failure: unknown): void => {
      if (current) {
        settle(signals, failure);
      }
    });
  return (): void => {
    current = false;
  };
};

/**
 * Runs `load` when `key` changes and on each retry.
 * A retry keeps the old value on screen until the new value arrives.
 * A null `load` means that there is nothing to load.
 */
export const useLoad = <T>(key: string, load: (() => Promise<T>) | null): Loaded<T> => {
  const value = useSignal<T | null>(null);
  const status = useSignal<LoadStatus>(load === null ? "ready" : "loading");
  const error = useSignal<string | null>(null);
  const attempt = useSignal(0);
  const loader = useRef(load);
  loader.current = load;
  useEffect((): (() => void) | undefined => {
    const run = loader.current;
    return run === null ? undefined : start(run, { value, status, error });
  }, [key, attempt.value, value, status, error]);
  return {
    value: value.value,
    status: status.value,
    error: error.value,
    retry: (): void => {
      attempt.value += 1;
    },
    set: (next: T): void => {
      value.value = next;
    },
  };
};
