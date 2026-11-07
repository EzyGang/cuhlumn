import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useLocation } from "wouter-preact";
import { useLoad } from "../../../hooks/useLoad.hook";
import { refreshSites } from "../../../hooks/useSites.hook";
import { ApiError } from "../../../services/http";
import { uploadMedia } from "../../../services/media.service";
import { deleteSite, getSite, saveSite } from "../../../services/sites.service";
import { draftsStore } from "../../../stores/drafts.store";
import type { Site } from "../../../types/api/Site";
import type { LoadState } from "../../../types/load";
import type { Navigate } from "../../../types/navigation";
import type { PendingImages, SiteForm } from "../../../types/siteSettings";
import { takeDraft } from "../../../utils/drafts";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { reportApiError, reportFormError } from "../../../utils/reportApiError";
import {
  formToSiteInput,
  refineFieldErrors,
  siteFieldPaths,
  siteToForm,
} from "../../../utils/siteForm";
import {
  IMAGE_LABELS,
  imageUploads,
  uploadPending,
  withUploadedImages,
} from "../../../utils/siteImages";
import type { ImagePaths } from "../../../utils/siteImages";
import { showToast } from "../../shared/ui/Toast.view";

interface SitePersistence {
  load: LoadState;
  saving: boolean;
  save: (event: Event) => void;
  remove: () => void;
}

interface SiteSignals {
  form: Signal<SiteForm>;
  errors: Signal<FieldErrors>;
  /** The image files that upload when the admin creates the site. */
  pending: Signal<PendingImages>;
}

interface SiteTarget extends SiteSignals {
  host: string | null;
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

/**
 * Uploads the image files of the new site, and saves their paths in the site.
 * A failed upload leaves its field empty and does not stop the other uploads.
 */
const attachImages = async (site: Site, images: PendingImages): Promise<ImagePaths> => {
  const outcomes = await uploadPending(images, (file) => uploadMedia(site.host, file));
  const { paths, failed } = imageUploads(outcomes);
  for (const failure of failed) {
    reportApiError(failure.error, `Cannot upload the ${IMAGE_LABELS[failure.key].toLowerCase()}`);
  }
  if (Object.keys(paths).length === 0) {
    return paths;
  }
  try {
    await saveSite(site.host, formToSiteInput({ ...siteToForm(site), ...paths }));
    return paths;
  } catch (error) {
    reportApiError(error, "Cannot save the images");
    return {};
  }
};

/** Opens the new site, and hands over text that was typed while the requests ran. */
const openCreated = (target: SiteTarget, site: Site, sent: SiteForm, paths: ImagePaths): void => {
  const { form, navigate } = target;
  if (form.value !== sent) {
    const draft = withUploadedImages(form.value, sent, paths);
    draftsStore.site.value = { key: site.host, form: draft, reason: "redirect" };
  }
  navigate(`/sites/${encodeURIComponent(site.host)}`, { replace: true });
};

const persist = async (target: SiteTarget): Promise<void> => {
  const { host, form, errors } = target;
  const sent = form.value;
  const images = target.pending.value;
  const name = host ?? sent.host.trim().toLowerCase().replace(/\.$/, "");
  const hostError = host === null ? await checkNewHost(name) : null;
  if (hostError !== null) {
    errors.value = { host: hostError };
    return;
  }
  const site = await saveSite(name, formToSiteInput(sent));
  errors.value = {};
  showToast("success", host === null ? "Site created" : "Site saved");
  if (host === null) {
    const paths = await attachImages(site, images);
    void refreshSites();
    openCreated(target, site, sent, paths);
    return;
  }
  void refreshSites();
  if (form.value === sent) {
    form.value = siteToForm(site);
  }
};

const save = async (target: SiteTarget, saving: Signal<boolean>): Promise<void> => {
  saving.value = true;
  try {
    await persist(target);
  } catch (error) {
    const known = siteFieldPaths(target.form.value);
    const fields = reportFormError(error, known, "Cannot save the site");
    target.errors.value = refineFieldErrors(fields, known);
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

export const useSitePersistence = (host: string | null, signals: SiteSignals): SitePersistence => {
  const [, navigate] = useLocation();
  const saving = useSignal(false);
  const load = useSiteLoader(host, signals.form);
  const target: SiteTarget = { host, ...signals, navigate };
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
