import { useSignal } from "@preact/signals";
import { useParams } from "wouter-preact";
import type { SiteForm } from "../../../types/siteSettings";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { MAX_LINKS, emptySiteForm, toColorInputValue } from "../../../utils/siteForm";
import type { SiteSettingsContextValue } from "../context/SiteSettingsContext";
import { useAnnouncement } from "./useAnnouncement.hook";
import { useAppearanceOptions } from "./useAppearanceOptions.hook";
import { useSiteFormState } from "./useSiteFormState.hook";
import { useSiteImages } from "./useSiteImages.hook";
import { useSitePersistence } from "./useSitePersistence.hook";
import { useSocialLinks } from "./useSocialLinks.hook";

type Data = SiteSettingsContextValue["data"];
type FormData = Pick<
  Data,
  "form" | "errors" | "picker" | "colorValue" | "images" | "linksTitle" | "canAddLink"
>;

/** Describes the form values that come straight from the form, its errors, and the picker. */
const formData = (
  form: SiteForm,
  errors: FieldErrors,
  images: Pick<FormData, "picker" | "images">,
): FormData => ({
  form,
  errors,
  picker: images.picker,
  colorValue: toColorInputValue(form.accent),
  images: images.images,
  linksTitle: `Header links (${form.links.length}/${MAX_LINKS})`,
  canAddLink: form.links.length < MAX_LINKS,
});

export const useSiteSettings = (): SiteSettingsContextValue => {
  const host = useParams<{ host?: string }>().host ?? null;
  const form = useSignal<SiteForm>(emptySiteForm());
  const errors = useSignal<FieldErrors>({});
  const { update, actions, ...state } = useSiteFormState(form, errors);
  const appearance = useAppearanceOptions(form.value, errors.value, update);
  const social = useSocialLinks(form, errors.value, update);
  const announcement = useAnnouncement(form, update);
  const images = useSiteImages(host, form.value, errors.value, actions.text);
  const persistence = useSitePersistence(host, { form, errors, pending: images.pending });
  return {
    data: {
      ...formData(form.value, errors.value, images),
      ...state,
      mode: host === null ? "create" : "edit",
      ...appearance.data,
      social: social.social,
      announcement: announcement.announcement,
    },
    status: { load: persistence.load, saving: persistence.saving },
    actions: {
      ...actions,
      ...appearance.actions,
      addSocial: social.addSocial,
      announcement: announcement.actions,
      save: persistence.save,
      remove: persistence.remove,
    },
  };
};
