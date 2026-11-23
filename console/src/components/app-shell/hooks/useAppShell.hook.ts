import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useLocation, useRoute } from "wouter-preact";
import { useSites } from "../../../hooks/useSites.hook";
import { deleteSession } from "../../../services/session.service";
import { sessionStore } from "../../../stores/session.store";
import { sitesStore } from "../../../stores/sites.store";
import type { AppShellModel } from "../../../types/appShell";
import type { Navigate } from "../../../types/navigation";
import { reportApiError } from "../../../utils/reportApiError";
import { siteHomePath, siteKind } from "../../../utils/sites";

const logOut = async (loggingOut: Signal<boolean>, navigate: Navigate): Promise<void> => {
  loggingOut.value = true;
  try {
    await deleteSession();
    sessionStore.status.value = "anonymous";
    sitesStore.sites.value = null;
    sitesStore.error.value = null;
    navigate("/login", { replace: true });
  } catch (error) {
    reportApiError(error, "Cannot log out");
  } finally {
    loggingOut.value = false;
  }
};

export const useAppShell = (): AppShellModel => {
  const [, navigate] = useLocation();
  const [, params] = useRoute<{ host: string }>("/sites/:host/*?");
  const { sites } = useSites();
  const loggingOut = useSignal(false);
  const host = params?.host ?? null;
  return {
    data: {
      hosts: (sites ?? []).map((site) => ({
        value: site.host,
        label: site.kind === "external" ? `${site.host} (external)` : site.host,
      })),
      host,
    },
    status: { loggingOut: loggingOut.value },
    actions: {
      switchSite: (next: string): void =>
        navigate(siteHomePath({ host: next, kind: siteKind(sites, next) ?? "blog" })),
      logout: (): void => void logOut(loggingOut, navigate),
    },
  };
};
