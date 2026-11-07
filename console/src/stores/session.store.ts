import { signal } from "@preact/signals";

/** `setup` means that the server has no admin token yet. */
export type SessionStatus = "checking" | "setup" | "authenticated" | "anonymous";

export const sessionStore = {
  status: signal<SessionStatus>("checking"),
};
