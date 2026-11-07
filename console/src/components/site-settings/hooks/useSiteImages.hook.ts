import type { Signal } from "@preact/signals";
import type {
  ImageFieldModel,
  ImageKey,
  MediaChoice,
  MediaPickerModel,
  PendingImages,
  SiteForm,
  SiteTextKey,
} from "../../../types/siteSettings";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { IMAGE_LABELS } from "../../../utils/siteImages";
import { useMediaPicker } from "./useMediaPicker.hook";
import { usePendingImages } from "./usePendingImages.hook";
import type { PendingImagesModel } from "./usePendingImages.hook";

type TextSetters = Record<SiteTextKey, (value: string) => void>;

interface SiteImages {
  picker: MediaPickerModel;
  images: Record<ImageKey, ImageFieldModel>;
  pending: Signal<PendingImages>;
}

interface ImageSources {
  form: SiteForm;
  errors: FieldErrors;
  setters: TextSetters;
  choices: readonly MediaChoice[];
  pending: PendingImagesModel;
}

/**
 * Returns the address to show as a thumbnail: the library address for a `/media/` path,
 * the value itself for an absolute address, and null otherwise.
 */
const thumbnailFor = (value: string, choices: readonly MediaChoice[]): string | null => {
  const url = value.trim();
  if (url.startsWith("/media/")) {
    return choices.find((choice) => choice.path === url)?.url ?? null;
  }
  return /^https?:\/\/[^/\s]+/i.test(url) ? url : null;
};

/** Describes one image field. A file that waits for the new site shows its name and preview. */
const imageField = (name: ImageKey, sources: ImageSources): ImageFieldModel => {
  const { form, setters, pending } = sources;
  const image = pending.pending.value[name];
  return {
    name,
    label: IMAGE_LABELS[name],
    value: image?.file.name ?? form[name],
    thumbnail: image === undefined ? thumbnailFor(form[name], sources.choices) : image.preview,
    error: sources.errors[`brand.${name}`],
    pending: image !== undefined,
    removeText: `Remove the ${IMAGE_LABELS[name].toLowerCase()} file`,
    setValue: (value: string): void => {
      pending.edit(name);
      setters[name](value);
    },
    removePending: (): void => pending.remove(name),
  };
};

/** Connects the image fields to the media picker and to the files that wait for a new site. */
export const useSiteImages = (
  host: string | null,
  form: SiteForm,
  errors: FieldErrors,
  setters: TextSetters,
): SiteImages => {
  const pending = usePendingImages();
  const stage = (key: ImageKey, file: File): void => {
    pending.choose(key, file);
    setters[key]("");
  };
  const picker = useMediaPicker(host, (key, path): void => setters[key](path), stage);
  const sources: ImageSources = { form, errors, setters, choices: picker.items, pending };
  return {
    picker,
    images: {
      logo: imageField("logo", sources),
      favicon: imageField("favicon", sources),
      og_image: imageField("og_image", sources),
    },
    pending: pending.pending,
  };
};
