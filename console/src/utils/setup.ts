import type { SessionStatus } from "../stores/session.store";

const ENTRY_PATHS: readonly string[] = ["/login", "/setup"];

/**
 * Returns the path that the console must go to for a session status, or null to stay.
 * A server without a token always sends the console to the setup page.
 */
export const redirectTarget = (status: SessionStatus, path: string): string | null => {
  if (status === "setup") {
    return path === "/setup" ? null : "/setup";
  }
  if (status === "anonymous") {
    return path === "/login" ? null : "/login";
  }
  if (status === "authenticated") {
    return ENTRY_PATHS.includes(path) ? "/" : null;
  }
  return null;
};
