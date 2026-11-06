import type { MediaItem } from "../types/api/MediaItem";
import type { MediaSaved } from "../types/api/MediaSaved";
import { requestEmpty, requestJson, segment } from "./http";

const mediaPath = (host: string): string => `/api/sites/${segment(host)}/media`;

export const listMedia = (host: string): Promise<MediaItem[]> =>
  requestJson<MediaItem[]>("GET", mediaPath(host));

export const uploadMedia = (host: string, file: File): Promise<MediaSaved> =>
  requestJson<MediaSaved>("POST", mediaPath(host), { body: file, contentType: file.type });

export const deleteMedia = (host: string, name: string): Promise<void> =>
  requestEmpty("DELETE", `${mediaPath(host)}/${segment(name)}`);
