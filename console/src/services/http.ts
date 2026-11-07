import { sessionStore } from "../stores/session.store";
import type { ErrorBody } from "../types/api/ErrorBody";
import type { ErrorCode } from "../types/api/ErrorCode";

/** Removes the `"<field>: "` prefix that the server puts before validation messages. */
export const stripFieldPrefix = (message: string, field: string | null): string =>
  field !== null && message.startsWith(`${field}: `) ? message.slice(field.length + 2) : message;

/** A failed API response. `message` holds the server message without the field prefix. */
export class ApiError extends Error {
  readonly status: number;
  readonly body: ErrorBody;

  constructor(status: number, body: ErrorBody) {
    super(stripFieldPrefix(body.error.message, body.error.field));
    this.name = "ApiError";
    this.status = status;
    this.body = body;
  }
}

interface RequestOptions {
  json?: unknown;
  body?: Blob;
  contentType?: string;
}

const errorBody = (code: ErrorCode, message: string): ErrorBody => ({
  error: { code, message, field: null },
});

const isErrorBody = (value: unknown): value is ErrorBody => {
  if (typeof value !== "object" || value === null || !("error" in value)) {
    return false;
  }
  const detail: unknown = value.error;
  return typeof detail === "object" && detail !== null && "message" in detail && "code" in detail;
};

const readError = async (response: Response): Promise<ApiError> => {
  const text = await response.text();
  try {
    const parsed: unknown = JSON.parse(text);
    if (isErrorBody(parsed)) {
      return new ApiError(response.status, parsed);
    }
  } catch {
    // A body that is not JSON gets a generic message below.
  }
  const message = `The server answered ${response.status} ${response.statusText}.`.trim();
  return new ApiError(response.status, errorBody("internal", message));
};

/** Reads a failed response, and sends the console to the setup page when the server has no token. */
const failure = async (response: Response): Promise<ApiError> => {
  const error = await readError(response);
  if (error.body.error.code === "needs_setup") {
    sessionStore.status.value = "setup";
  }
  return error;
};

const send = async (method: string, path: string, options: RequestOptions): Promise<Response> => {
  const headers = new Headers({ Accept: "application/json" });
  let body: BodyInit | undefined;
  if (options.json !== undefined) {
    headers.set("Content-Type", "application/json");
    body = JSON.stringify(options.json);
  } else if (options.body !== undefined) {
    headers.set("Content-Type", options.contentType ?? "application/octet-stream");
    body = options.body;
  }
  let response: Response;
  try {
    response = await fetch(path, {
      method,
      headers,
      body: body ?? null,
      credentials: "same-origin",
    });
  } catch {
    throw new ApiError(0, errorBody("internal", "The server cannot be reached."));
  }
  if (!response.ok) {
    throw await failure(response);
  }
  return response;
};

export const requestJson = async <T>(
  method: string,
  path: string,
  options: RequestOptions = {},
): Promise<T> => {
  const response = await send(method, path, options);
  const data: T = await response.json();
  return data;
};

export const requestEmpty = async (
  method: string,
  path: string,
  options: RequestOptions = {},
): Promise<void> => {
  await send(method, path, options);
};

export const segment = (value: string): string => encodeURIComponent(value);
