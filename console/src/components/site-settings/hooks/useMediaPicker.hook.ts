import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useLoad } from "../../../hooks/useLoad.hook";
import { listMedia } from "../../../services/media.service";
import type { MediaItem } from "../../../types/api/MediaItem";
import type { ImageKey, MediaChoice, MediaPickerModel } from "../../../types/siteSettings";
import { bindKeys } from "../../../utils/bindKeys";

type PickImage = (key: ImageKey, path: string) => void;

const IMAGE_KEYS: readonly ImageKey[] = ["logo", "favicon", "og_image"];

/** Returns the root-relative path of an uploaded image, which works behind any proxy or port. */
const mediaPath = (item: MediaItem): string => `/media/${item.name}`;

const toChoice = (
  item: MediaItem,
  target: Signal<ImageKey | null>,
  pick: PickImage,
): MediaChoice => ({
  name: item.name,
  url: item.url,
  path: mediaPath(item),
  pick: (): void => {
    const key = target.value;
    target.value = null;
    if (key !== null) {
      pick(key, mediaPath(item));
    }
  },
});

/** Loads the media library of the site, and opens it in a dialog for one image field. */
export const useMediaPicker = (host: string | null, pick: PickImage): MediaPickerModel => {
  const target = useSignal<ImageKey | null>(null);
  const library = useLoad(
    host ?? "",
    host === null ? null : (): Promise<MediaItem[]> => listMedia(host),
  );
  const setOpen = (key: ImageKey, open: boolean): void => {
    target.value = open ? key : null;
    if (open) {
      library.retry();
    }
  };
  return {
    target: target.value,
    items: (library.value ?? []).map((item) => toChoice(item, target, pick)),
    loading: library.status === "loading",
    error: library.status === "failed" ? library.error : null,
    available: host !== null,
    setOpen: bindKeys(IMAGE_KEYS, setOpen),
  };
};
