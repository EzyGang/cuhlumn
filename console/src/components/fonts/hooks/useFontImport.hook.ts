import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { importGoogleFont, uploadFont } from "../../../services/fonts.service";
import type { FontImportModel } from "../../../types/fonts";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { reportApiError, reportFormError } from "../../../utils/reportApiError";
import { showToast } from "../../shared/ui/Toast.view";

interface ImportState {
  url: Signal<string>;
  errors: Signal<FieldErrors>;
  importing: Signal<boolean>;
}

/** Uploads the files one after another, then loads the library again. */
const uploadAll = async (
  files: readonly File[],
  uploading: Signal<boolean>,
  reload: () => void,
): Promise<void> => {
  uploading.value = true;
  for (const file of files) {
    try {
      const font = await uploadFont(file);
      showToast("success", "Font uploaded", font.families.join(", "));
    } catch (error) {
      reportApiError(error, `Cannot upload ${file.name}`);
    }
  }
  uploading.value = false;
  reload();
};

const importFrom = async (state: ImportState, reload: () => void): Promise<void> => {
  state.importing.value = true;
  try {
    const fonts = await importGoogleFont({ url: state.url.value.trim() });
    const families = [...new Set(fonts.flatMap((font) => font.families))];
    const title =
      fonts.length === 1 ? "1 font file imported" : `${fonts.length} font files imported`;
    showToast("success", title, families.join(", "));
    state.url.value = "";
    state.errors.value = {};
    reload();
  } catch (error) {
    state.errors.value = reportFormError(error, ["url"], "Cannot import the font");
  } finally {
    state.importing.value = false;
  }
};

export const useFontImport = (reload: () => void): FontImportModel => {
  const uploading = useSignal(false);
  const state: ImportState = {
    url: useSignal(""),
    errors: useSignal({}),
    importing: useSignal(false),
  };
  return {
    data: { googleUrl: state.url.value, urlError: state.errors.value["url"] },
    status: { uploading: uploading.value, importing: state.importing.value },
    actions: {
      pickFiles: (event: Event): void => {
        const input = event.currentTarget as HTMLInputElement;
        void uploadAll([...(input.files ?? [])], uploading, reload);
        input.value = "";
      },
      setGoogleUrl: (url: string): void => {
        state.url.value = url;
      },
      importGoogle: (event: Event): void => {
        event.preventDefault();
        void importFrom(state, reload);
      },
    },
  };
};
