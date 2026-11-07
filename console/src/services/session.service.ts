import type { Session } from "../types/api/Session";
import type { SessionInput } from "../types/api/SessionInput";
import type { TokenInput } from "../types/api/TokenInput";
import { requestEmpty, requestJson } from "./http";

export const getSession = (): Promise<Session> => requestJson<Session>("GET", "/api/session");

export const createSession = (input: SessionInput): Promise<Session> =>
  requestJson<Session>("POST", "/api/session", { json: input });

export const deleteSession = (): Promise<void> => requestEmpty("DELETE", "/api/session");

/** Sets the first admin token of the server and opens a session. */
export const setUp = (input: TokenInput): Promise<Session> =>
  requestJson<Session>("POST", "/api/setup", { json: input });
