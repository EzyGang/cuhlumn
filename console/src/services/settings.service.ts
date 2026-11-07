import type { ServerSettings } from "../types/api/ServerSettings";
import type { ServerSettingsInput } from "../types/api/ServerSettingsInput";
import type { Session } from "../types/api/Session";
import type { TokenInput } from "../types/api/TokenInput";
import { requestJson } from "./http";

export const getSettings = (): Promise<ServerSettings> =>
  requestJson<ServerSettings>("GET", "/api/settings");

export const saveSettings = (input: ServerSettingsInput): Promise<ServerSettings> =>
  requestJson<ServerSettings>("PUT", "/api/settings", { json: input });

export const rotateToken = (input: TokenInput): Promise<Session> =>
  requestJson<Session>("PUT", "/api/settings/token", { json: input });
