import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useLocation } from "wouter-preact";
import { useLoad } from "../../../hooks/useLoad.hook";
import { refreshSites } from "../../../hooks/useSites.hook";
import { ApiError } from "../../../services/http";
import { deleteSite, getSite, saveSite } from "../../../services/sites.service";
import { draftsStore } from "../../../stores/drafts.store";
import type { Site } from "../../../types/api/Site";
import type { LoadState } from "../../../types/load";
import type { Navigate } from "../../../types/navigation";
import type { SiteForm } from "../../../types/siteSettings";
import { takeDraft } from "../../../utils/drafts";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { reportApiError, reportFormError } from "../../../utils/reportApiError";
import { formToSiteInput, siteFieldPaths, siteToForm } from "../../../utils/siteForm";
import { showToast } from "../../shared/ui/Toast.view";

interface SitePersistence {
  load: LoadState;
  saving: boolean;
  save: (event: Event) => void;
  remove: () => void;
}

interface SiteTarget {
  host: string | null;
  form: Signal<SiteForm>;
  errors: Signal<FieldErrors>;
  navigate: Navigate;
}

/** Returns an error for the new host, or null when no site uses the host yet. */
const checkNewHost = async (host: string): Promise<string | null> => {
  if (host === "") {
    return "Enter the domain of the site.";
  }
  try {
    await getSite(host);
    return "A site with this domain exists already.";
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) {
      return null;
    }
    if (error instanceof ApiError && error.status === 400) {
      return error.message;
    }
    throw error;
  }
};

/** Opens the new site, and hands over text that was typed while the request ran. */
const openCreated = ({ form, navigate }: SiteTarget, site: Site, sent: SiteForm): void => {
  if (form.value !== sent) {
    draftsStore.site.value = { key: site.host, form: form.value, reason: "redirect" };
  }
  navigate(`/sites/${encodeURIComponent(site.host)}`, { replace: true });
};

const persist = async (target: SiteTarget): Promise<void> => {
  const { host, form, errors } = target;
  const sent = form.value;
  const name = host ?? sent.host.trim().toLowerCase().replace(/\.$/, "");
  const hostError = host === null ? await checkNewHost(name) : null;
  if (hostError !== null) {
    errors.value = { host: hostError };
    return;
  }
  const site = await saveSite(name, formToSiteInput(sent));
  errors.value = {};
  showToast("success", host === null ? "Site created" : "Site saved");
  void refreshSites();
  if (host === null) {
    openCreated(target, site, sent);
  } else if (form.value === sent) {
    form.value = siteToForm(site);
  }
};

const save = async (target: SiteTarget, saving: Signal<boolean>): Promise<void> => {
  saving.value = true;
  try {
    await persist(target);
  } catch (error) {
    const known = siteFieldPaths(target.form.value);
    target.errors.value = reportFormError(error, known, "Cannot save the site");
  } finally {
    saving.value = false;
  }
};

const remove = async ({ host, navigate }: SiteTarget): Promise<void> => {
  if (host === null) {
    return;
  }
  try {
    await deleteSite(host);
    await refreshSites();
    showToast("success", "Site deleted", host);
    navigate("/", { replace: true });
  } catch (error) {
    reportApiError(error, "Cannot delete the site");
  }
};

const useSiteLoader = (host: string | null, form: Signal<SiteForm>): LoadState => {
  const load = async (name: string): Promise<Site> => {
    const site = await getSite(name);
    form.value = takeDraft(draftsStore.site, name)?.form ?? siteToForm(site);
    return site;
  };
  const loaded = useLoad(host ?? "", host === null ? null : (): Promise<Site> => load(host));
  return { status: loaded.status, error: loaded.error, retry: loaded.retry };
};

export const useSitePersistence = (
  host: string | null,
  form: Signal<SiteForm>,
  errors: Signal<FieldErrors>,
): SitePersistence => {
  const [, navigate] = useLocation();
  const saving = useSignal(false);
  const load = useSiteLoader(host, form);
  const target: SiteTarget = { host, form, errors, navigate };
  return {
    load,
    saving: saving.value,
    save: (event: Event): void => {
      event.preventDefault();
      void save(target, saving);
    },
    remove: (): void => void remove(target),
  };
};
