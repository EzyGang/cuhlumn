import type { Health } from "../types/api/Health";
import { requestJson } from "./http";

export const getHealth = (): Promise<Health> => requestJson<Health>("GET", "/api/health");
