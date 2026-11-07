import type { LoadState } from "./load";

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
