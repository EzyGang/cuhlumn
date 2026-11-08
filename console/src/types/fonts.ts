import type { Font } from "./api/Font";
import type { LoadState } from "./load";
import type { RadioCardItem } from "./radioCards";

export interface FontRow {
  name: string;
  families: string;
  source: string;
  size: string;
  created: string;
  remove: () => void;
}

export interface FontImportModel {
  data: { googleUrl: string; urlError: string | undefined };
  status: { uploading: boolean; importing: boolean };
  actions: {
    pickFiles: (event: Event) => void;
    setGoogleUrl: (url: string) => void;
    importGoogle: (event: Event) => void;
  };
}

export interface FontsModel {
  data: FontImportModel["data"] & { rows: FontRow[]; accept: string };
  status: FontImportModel["status"] & { load: LoadState };
  actions: FontImportModel["actions"];
}

/** A Font control value. The prefix keeps a library family apart from the system options. */
export type FontValue = `system:${Font}` | `family:${string}`;

/** The items of the Font control and the selected value. */
export interface FontChoice {
  items: RadioCardItem<FontValue>[];
  value: FontValue;
  /** True when a library family is selected, so the fallback choice shows. */
  usesFamily: boolean;
}

/** The Font control in the Layout group of the site settings. */
export interface FontModel extends FontChoice {
  /** A short message when the font library cannot load, else null. */
  note: string | null;
  error: string | undefined;
  manageHref: string;
}
