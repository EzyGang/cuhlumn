import type { FontItem } from "../types/api/FontItem";
import type { GoogleFontInput } from "../types/api/GoogleFontInput";
import { requestEmpty, requestJson, segment } from "./http";

export const listFonts = (): Promise<FontItem[]> => requestJson<FontItem[]>("GET", "/api/fonts");

export const uploadFont = (file: File): Promise<FontItem> =>
  requestJson<FontItem>("POST", "/api/fonts", {
    body: file,
    contentType: file.type === "" ? "application/octet-stream" : file.type,
  });

export const importGoogleFont = (input: GoogleFontInput): Promise<FontItem[]> =>
  requestJson<FontItem[]>("POST", "/api/fonts/google", { json: input });

export const deleteFont = (name: string): Promise<void> =>
  requestEmpty("DELETE", `/api/fonts/${segment(name)}`);
