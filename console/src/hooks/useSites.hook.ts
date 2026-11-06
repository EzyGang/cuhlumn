import { useEffect } from "preact/hooks";
import { listSites } from "../services/sites.service";
import { sitesStore } from "../stores/sites.store";
import type { Site } from "../types/api/Site";
import type { LoadState } from "../types/load";
import { errorMessage, isUnauthorized, reportApiError } from "../utils/reportApiError";

interface SitesModel {
  sites: Site[] | null;
  load: LoadState;
}

let latestRequest = 0;

/** Loads the site list into the sites store again. Only the newest request writes its answer. */
export const refreshSites = async (): Promise<void> => {
  latestRequest += 1;
  const request = latestRequest;
  try {
    const sites = await listSites();
    if (request === latestRequest) {
      sitesStore.sites.value = sites;
      sitesStore.error.value = null;
    }
  } catch (error) {
    if (request !== latestRequest) {
      return;
    }
    if (isUnauthorized(error)) {
      reportApiError(error);
    }
    sitesStore.error.value = errorMessage(error);
  }
};

const retry = (): void => {
  sitesStore.error.value = null;
  void refreshSites();
};

/** Returns the site list, and loads it when the store is empty. */
export const useSites = (): SitesModel => {
  const sites = sitesStore.sites.value;
  const error = sitesStore.error.value;
  useEffect((): void => {
    if (sitesStore.sites.value === null) {
      void refreshSites();
    }
  }, []);
  const status = sites !== null ? "ready" : error === null ? "loading" : "failed";
  return { sites, load: { status, error, retry } };
};
