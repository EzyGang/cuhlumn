import { signal } from "@preact/signals";
import type { Site } from "../types/api/Site";

export const sitesStore = {
  sites: signal<Site[] | null>(null),
  error: signal<string | null>(null),
};
