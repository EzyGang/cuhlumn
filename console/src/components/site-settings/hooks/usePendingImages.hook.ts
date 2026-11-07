import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import type { ImageKey, PendingImages } from "../../../types/siteSettings";
import { changePending } from "../../../utils/siteImages";
import type { PendingAction } from "../../../utils/siteImages";

export interface PendingImagesModel {
  pending: Signal<PendingImages>;
  choose: (key: ImageKey, file: File) => void;
  remove: (key: ImageKey) => void;
  edit: (key: ImageKey) => void;
}

/** Keeps the image files that upload when the admin creates the site, with local previews. */
export const usePendingImages = (): PendingImagesModel => {
  const pending = useSignal<PendingImages>({});
  const apply = (action: PendingAction): void => {
    pending.value = changePending(pending.value, action);
  };
  const choose = (key: ImageKey, file: File): void => {
    apply({ type: "choose", key, file });
    // The preview is a `data:` address, because the console Content Security Policy allows it.
    // If the browser cannot read the file, the thumbnail stays empty and the file still uploads.
    const reader = new FileReader();
    reader.onload = (): void => {
      apply({ type: "preview", key, file, preview: String(reader.result) });
    };
    reader.readAsDataURL(file);
  };
  return {
    pending,
    choose,
    remove: (key: ImageKey): void => apply({ type: "remove", key }),
    edit: (key: ImageKey): void => {
      if (pending.value[key] !== undefined) {
        apply({ type: "edit", key });
      }
    },
  };
};
