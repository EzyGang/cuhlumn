import type { Kind } from "./api/Kind";
import type { TocEntry } from "./api/TocEntry";

export type PostTextKey = "slug" | "title" | "description" | "body" | "cover" | "og_image";

export interface PostForm extends Record<PostTextKey, string> {
  tags: string[];
  kind: Kind;
}

export type PreviewState =
  | { kind: "empty" }
  | { kind: "loading" }
  | { kind: "ready"; html: string; toc: TocEntry[]; wordCount: number }
  | { kind: "unavailable" }
  | { kind: "failed"; message: string };

export interface TocItem {
  id: string;
  label: string;
  indent: string;
}

export interface PreviewModel {
  message: string | null;
  document: string | null;
  toc: TocItem[];
  wordCount: string | null;
}
