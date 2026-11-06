import { ApiError } from "../services/http";

export type FieldErrors = Readonly<Record<string, string>>;

export interface FormErrors {
  fields: FieldErrors;
  banner: string | null;
}

const UNKNOWN_FAILURE = "The request failed for an unknown reason.";

export const normalizeFieldPath = (path: string): string =>
  path
    .replaceAll(/\[(\d+)\]/g, ".$1")
    .replace(/^\./, "")
    .trim();

const closestField = (path: string, known: ReadonlySet<string>): string | null => {
  const parts = normalizeFieldPath(path).split(".");
  for (let length = parts.length; length > 0; length -= 1) {
    const candidate = parts.slice(0, length).join(".");
    if (known.has(candidate)) {
      return candidate;
    }
  }
  return null;
};

export const mapErrorToFields = (error: unknown, known: readonly string[]): FormErrors => {
  if (!(error instanceof ApiError)) {
    return { fields: {}, banner: error instanceof Error ? error.message : UNKNOWN_FAILURE };
  }
  const { field } = error.body.error;
  const match = field === null ? null : closestField(field, new Set(known));
  if (match === null) {
    return { fields: {}, banner: error.message };
  }
  return { fields: { [match]: error.message }, banner: null };
};

/** Removes the error of a field and the errors of its list items. */
export const withoutFieldError = (errors: FieldErrors, path: string): FieldErrors =>
  Object.fromEntries(
    Object.entries(errors).filter(([key]) => key !== path && !key.startsWith(`${path}.`)),
  );

/** Returns the index and message of the first list item error, or the error of the whole list. */
export const firstItemError = (
  errors: FieldErrors,
  path: string,
): [number | null, string | undefined] => {
  const entry = Object.entries(errors).find(([key]) => key.startsWith(`${path}.`));
  return entry === undefined
    ? [null, errors[path]]
    : [Number(entry[0].slice(path.length + 1)), entry[1]];
};
