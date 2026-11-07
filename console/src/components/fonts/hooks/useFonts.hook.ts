import { useLoad } from "../../../hooks/useLoad.hook";
import type { Loaded } from "../../../hooks/useLoad.hook";
import { deleteFont, listFonts } from "../../../services/fonts.service";
import type { FontItem } from "../../../types/api/FontItem";
import type { FontRow, FontsModel } from "../../../types/fonts";
import { formatBytes, formatDateTime } from "../../../utils/format";
import { reportApiError } from "../../../utils/reportApiError";
import { showToast } from "../../shared/ui/Toast.view";
import { useFontImport } from "./useFontImport.hook";

type Library = Loaded<FontItem[]>;

const ACCEPT = ".ttf,.otf,.ttc,font/ttf,font/otf,font/collection";

const SOURCES: Record<FontItem["source"], string> = { upload: "Upload", google: "Google Fonts" };

const removeFrom = async (name: string, library: Library): Promise<void> => {
  try {
    await deleteFont(name);
    library.set((library.value ?? []).filter((item) => item.name !== name));
    showToast("success", "Font deleted", name);
  } catch (error) {
    reportApiError(error, "Cannot delete the font");
  }
};

const toRow = (item: FontItem, library: Library): FontRow => ({
  name: item.name,
  families: item.families.join(", "),
  source: SOURCES[item.source],
  size: formatBytes(item.size),
  created: formatDateTime(item.created_at),
  remove: (): void => void removeFrom(item.name, library),
});

export const useFonts = (): FontsModel => {
  const library = useLoad("fonts", listFonts);
  const imports = useFontImport(library.retry);
  return {
    data: {
      ...imports.data,
      rows: (library.value ?? []).map((item) => toRow(item, library)),
      accept: ACCEPT,
    },
    status: {
      ...imports.status,
      load: { status: library.status, error: library.error, retry: library.retry },
    },
    actions: imports.actions,
  };
};
