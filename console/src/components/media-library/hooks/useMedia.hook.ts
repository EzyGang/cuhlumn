import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useParams } from "wouter-preact";
import { useLoad } from "../../../hooks/useLoad.hook";
import type { Loaded } from "../../../hooks/useLoad.hook";
import { deleteMedia, listMedia, uploadMedia } from "../../../services/media.service";
import type { MediaItem } from "../../../types/api/MediaItem";
import type { MediaModel, MediaTile } from "../../../types/media";
import { formatBytes, formatDateTime } from "../../../utils/format";
import { MEDIA_ACCEPT } from "../../../utils/media";
import { reportApiError } from "../../../utils/reportApiError";
import { showToast } from "../../shared/ui/Toast.view";

type Library = Loaded<MediaItem[]>;

const DROP_TEXT = "Drop PNG, JPEG, WebP, GIF, or AVIF images here, up to 16 MiB each.";

const copyAddress = async (url: string): Promise<void> => {
  try {
    await navigator.clipboard.writeText(url);
    showToast("success", "Address copied", url);
  } catch {
    showToast("error", "Cannot copy the address", url);
  }
};

const removeFrom = async (host: string, name: string, library: Library): Promise<void> => {
  try {
    await deleteMedia(host, name);
    library.set((library.value ?? []).filter((item) => item.name !== name));
    showToast("success", "Image deleted", name);
  } catch (error) {
    reportApiError(error, "Cannot delete the image");
  }
};

const toTile = (host: string, item: MediaItem, library: Library): MediaTile => ({
  name: item.name,
  url: item.url,
  details: `${item.content_type.replace("image/", "").toUpperCase()} · ${formatBytes(item.size)}`,
  created: formatDateTime(item.created_at),
  copy: (): void => void copyAddress(item.url),
  remove: (): void => void removeFrom(host, item.name, library),
});

/** Uploads the files one after another. Files that arrive during an upload are refused. */
const uploadAll = async (
  host: string,
  files: readonly File[],
  library: Library,
  uploading: Signal<boolean>,
): Promise<void> => {
  if (uploading.value) {
    showToast("error", "An upload is running", "Add more images when it ends.");
    return;
  }
  uploading.value = true;
  for (const file of files) {
    try {
      await uploadMedia(host, file);
      showToast("success", "Image uploaded", file.name);
    } catch (error) {
      reportApiError(error, `Cannot upload ${file.name}`);
    }
  }
  uploading.value = false;
  library.retry();
};

interface UploadModel {
  dragging: boolean;
  uploading: boolean;
  actions: MediaModel["actions"];
}

const useUpload = (host: string, library: Library): UploadModel => {
  const dragging = useSignal(false);
  const uploading = useSignal(false);
  const upload = (files: readonly File[]): void => void uploadAll(host, files, library, uploading);
  return {
    dragging: dragging.value,
    uploading: uploading.value,
    actions: {
      pickFiles: (event: Event): void => {
        const input = event.currentTarget as HTMLInputElement;
        upload([...(input.files ?? [])]);
        input.value = "";
      },
      dragOver: (event: DragEvent): void => {
        event.preventDefault();
        dragging.value = !uploading.value;
      },
      dragLeave: (): void => {
        dragging.value = false;
      },
      drop: (event: DragEvent): void => {
        event.preventDefault();
        dragging.value = false;
        upload([...(event.dataTransfer?.files ?? [])]);
      },
    },
  };
};

export const useMedia = (): MediaModel => {
  const host = useParams<{ host: string }>().host;
  const library = useLoad(host, (): Promise<MediaItem[]> => listMedia(host));
  const upload = useUpload(host, library);
  return {
    data: {
      tiles: (library.value ?? []).map((item) => toTile(host, item, library)),
      dragging: upload.dragging,
      dropText: upload.uploading ? "Uploading" : DROP_TEXT,
      accept: MEDIA_ACCEPT,
    },
    status: {
      load: { status: library.status, error: library.error, retry: library.retry },
      uploading: upload.uploading,
    },
    actions: upload.actions,
  };
};
