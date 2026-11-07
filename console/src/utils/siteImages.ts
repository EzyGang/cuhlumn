import type { ImageKey, PendingImages, SiteForm } from "../types/siteSettings";
import { mediaPath } from "./media";

export const IMAGE_KEYS: readonly ImageKey[] = ["logo", "favicon", "og_image"];

export const IMAGE_LABELS: Record<ImageKey, string> = {
  logo: "Logo",
  favicon: "Favicon",
  og_image: "Default image",
};

/**
 * A change to the images that wait for a new site.
 * `choose` stores a file for the field without a preview and replaces the old file.
 * `preview` adds the preview of a file when the field still holds that file.
 * `remove` deletes the file of the field.
 * `edit` means that the admin typed an address in the field, which also deletes the file.
 */
export type PendingAction =
  | { type: "choose"; key: ImageKey; file: File }
  | { type: "preview"; key: ImageKey; file: File; preview: string }
  | { type: "remove"; key: ImageKey }
  | { type: "edit"; key: ImageKey };

export interface FailedUpload {
  key: ImageKey;
  file: File;
  error: unknown;
}

export type UploadOutcome = { key: ImageKey; file: File; name: string } | FailedUpload;

export type ImagePaths = Partial<Record<ImageKey, string>>;

export interface ImageUploads {
  /** The `/media/` path of each uploaded image. */
  paths: ImagePaths;
  /** The uploads that failed. Their fields stay empty. */
  failed: FailedUpload[];
}

/** Applies one change to the pending images. */
export const changePending = (pending: PendingImages, action: PendingAction): PendingImages => {
  switch (action.type) {
    case "choose":
      return { ...pending, [action.key]: { file: action.file, preview: null } };
    case "preview":
      return pending[action.key]?.file === action.file
        ? { ...pending, [action.key]: { file: action.file, preview: action.preview } }
        : pending;
    case "remove":
    case "edit":
      return Object.fromEntries(Object.entries(pending).filter(([key]) => key !== action.key));
  }
};

/** Uploads the pending images one after another, and continues after a failure. */
export const uploadPending = async (
  pending: PendingImages,
  upload: (file: File) => Promise<{ name: string }>,
): Promise<UploadOutcome[]> => {
  const outcomes: UploadOutcome[] = [];
  for (const key of IMAGE_KEYS) {
    const image = pending[key];
    if (image === undefined) {
      continue;
    }
    try {
      const saved = await upload(image.file);
      outcomes.push({ key, file: image.file, name: saved.name });
    } catch (error) {
      outcomes.push({ key, file: image.file, error });
    }
  }
  return outcomes;
};

/** Gives each uploaded image its `/media/` path, and collects the failed uploads. */
export const imageUploads = (outcomes: readonly UploadOutcome[]): ImageUploads => {
  const uploads: ImageUploads = { paths: {}, failed: [] };
  for (const outcome of outcomes) {
    if ("name" in outcome) {
      uploads.paths[outcome.key] = mediaPath(outcome.name);
    } else {
      uploads.failed.push(outcome);
    }
  }
  return uploads;
};

/**
 * Puts the uploaded image paths into a form that the admin changed during the save.
 * A field that the admin changed after the save started keeps the new value.
 */
export const withUploadedImages = (form: SiteForm, sent: SiteForm, paths: ImagePaths): SiteForm => {
  const next = { ...form };
  for (const key of IMAGE_KEYS) {
    const path = paths[key];
    if (path !== undefined && form[key] === sent[key]) {
      next[key] = path;
    }
  }
  return next;
};
