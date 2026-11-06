import { showToast } from "../components/shared/ui/Toast.view";
import { ApiError } from "../services/http";
import { sessionStore } from "../stores/session.store";
import { sitesStore } from "../stores/sites.store";
import { mapErrorToFields } from "./fieldErrors";
import type { FieldErrors } from "./fieldErrors";

export const isUnauthorized = (error: unknown): boolean =>
  error instanceof ApiError && error.status === 401;

/** Returns the text to show for a failure. */
export const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

/** Ends the session on a 401 response, and shows every other failure in a toast. */
export const reportApiError = (error: unknown, title = "Request failed"): void => {
  if (isUnauthorized(error)) {
    sessionStore.status.value = "anonymous";
    sitesStore.sites.value = null;
    sitesStore.error.value = null;
    return;
  }
  const field = error instanceof ApiError ? error.body.error.field : null;
  showToast("error", field === null ? title : `${title}: ${field}`, errorMessage(error));
};

/** Returns the field errors of a failed form request, and reports the other failures. */
export const reportFormError = (
  error: unknown,
  known: readonly string[],
  title: string,
): FieldErrors => {
  const mapped = mapErrorToFields(error, known);
  if (mapped.banner !== null) {
    reportApiError(error, title);
  }
  return mapped.fields;
};
