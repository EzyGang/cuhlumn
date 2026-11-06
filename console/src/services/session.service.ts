import type { Session } from "../types/api/Session";
import type { SessionInput } from "../types/api/SessionInput";
import { requestEmpty, requestJson } from "./http";

export const getSession = (): Promise<Session> => requestJson<Session>("GET", "/api/session");

export const createSession = (input: SessionInput): Promise<Session> =>
  requestJson<Session>("POST", "/api/session", { json: input });

export const deleteSession = (): Promise<void> => requestEmpty("DELETE", "/api/session");
