import { useEffect } from "preact/hooks";
import { getHealth } from "../services/health.service";
import { getSession } from "../services/session.service";
import { sessionStore } from "../stores/session.store";
import { isUnauthorized, reportApiError } from "../utils/reportApiError";

/** Asks whether the server needs the first setup, and else whether the browser has a session. */
const checkSession = async (): Promise<void> => {
  const health = await getHealth();
  if (health.needs_setup) {
    sessionStore.status.value = "setup";
    return;
  }
  await getSession();
  sessionStore.status.value = "authenticated";
};

/** Runs the startup check once. */
export const useSessionCheck = (): void => {
  useEffect((): void => {
    checkSession().catch((error: unknown): void => {
      if (sessionStore.status.value === "setup") {
        return;
      }
      sessionStore.status.value = "anonymous";
      if (!isUnauthorized(error)) {
        reportApiError(error, "Session check failed");
      }
    });
  }, []);
};
