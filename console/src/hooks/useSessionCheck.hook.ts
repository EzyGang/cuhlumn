import { useEffect } from "preact/hooks";
import { getSession } from "../services/session.service";
import { sessionStore } from "../stores/session.store";
import { isUnauthorized, reportApiError } from "../utils/reportApiError";

/** Asks the server once at startup whether the browser has a valid session. */
export const useSessionCheck = (): void => {
  useEffect((): void => {
    getSession()
      .then((): void => {
        sessionStore.status.value = "authenticated";
      })
      .catch((error: unknown): void => {
        sessionStore.status.value = "anonymous";
        if (!isUnauthorized(error)) {
          reportApiError(error, "Session check failed");
        }
      });
  }, []);
};
