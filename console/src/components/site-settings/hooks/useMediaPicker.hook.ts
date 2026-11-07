import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useLoad } from "../../../hooks/useLoad.hook";
import type { Loaded } from "../../../hooks/useLoad.hook";
import { listMedia, uploadMedia } from "../../../services/media.service";
import type { MediaItem } from "../../../types/api/MediaItem";
import type { ImageKey, MediaChoice, MediaPickerModel } from "../../../types/siteSettings";
import { bindKeys } from "../../../utils/bindKeys";
import { MEDIA_ACCEPT, mediaPath } from "../../../utils/media";
import { reportApiError } from "../../../utils/reportApiError";
import { IMAGE_KEYS } from "../../../utils/siteImages";
import { showToast } from "../../shared/ui/Toast.view";

type PickImage = (key: ImageKey, path: string) => void;
type StageImage = (key: ImageKey, file: File) => void;

interface PickerState {
  target: Signal<ImageKey | null>;
  uploading: Signal<boolean>;
  library: Loaded<MediaItem[]>;
  pick: PickImage;
}

const toChoice = (
  item: MediaItem,
  target: Signal<ImageKey | null>,
  pick: PickImage,
): MediaChoice => ({
  name: item.name,
  url: item.url,
  path: mediaPath(item.name),
  pick: (): void => {
    const key = target.value;
    target.value = null;
    if (key !== null) {
      pick(key, mediaPath(item.name));
    }
  },
});

/** Uploads the file to the library, puts it into the field, and closes the dialog. */
const uploadNow = async (
  host: string,
  key: ImageKey,
  file: File,
  state: PickerState,
): Promise<void> => {
  state.uploading.value = true;
  try {
    const saved = await uploadMedia(host, file);
    showToast("success", "Image uploaded", file.name);
    state.pick(key, mediaPath(saved.name));
    state.library.retry();
    if (state.target.value === key) {
      state.target.value = null;
    }
  } catch (error) {
    reportApiError(error, `Cannot upload ${file.name}`);
  } finally {
    state.uploading.value = false;
  }
};

/**
 * Returns the handler of the upload input.
 * A saved site uploads the file at once. A new site keeps the file until the admin creates the site.
 */
const uploadHandler =
  (host: string | null, state: PickerState, stage: StageImage): ((event: Event) => void) =>
  (event: Event): void => {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    const key = state.target.value;
    if (file === undefined || key === null) {
      return;
    }
    if (host === null) {
      stage(key, file);
      state.target.value = null;
      return;
    }
    void uploadNow(host, key, file, state);
  };

/** Opens the media library of the site in a dialog for one image field, with an upload action. */
export const useMediaPicker = (
  host: string | null,
  pick: PickImage,
  stage: StageImage,
): MediaPickerModel => {
  const target = useSignal<ImageKey | null>(null);
  const uploading = useSignal(false);
  const library = useLoad(
    host ?? "",
    host === null ? null : (): Promise<MediaItem[]> => listMedia(host),
  );
  const setOpen = (key: ImageKey, open: boolean): void => {
    target.value = open ? key : null;
    if (open && host !== null) {
      library.retry();
    }
  };
  return {
    target: target.value,
    items: (library.value ?? []).map((item) => toChoice(item, target, pick)),
    loading: library.status === "loading",
    error: library.status === "failed" ? library.error : null,
    library: host !== null,
    uploading: uploading.value,
    accept: MEDIA_ACCEPT,
    setOpen: bindKeys(IMAGE_KEYS, setOpen),
    upload: uploadHandler(host, { target, uploading, library, pick }, stage),
  };
};
