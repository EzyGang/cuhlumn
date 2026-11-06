import { signal } from "@preact/signals";

export type SessionStatus = "checking" | "authenticated" | "anonymous";

export const sessionStore = {
  status: signal<SessionStatus>("checking"),
};
