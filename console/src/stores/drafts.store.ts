import { signal } from "@preact/signals";
import type { PostForm } from "../types/postEditor";
import type { SiteForm } from "../types/siteSettings";

/** Why the editor handed its unsaved form over. */
export type DraftReason = "redirect" | "session";

export interface Draft<F> {
  key: string;
  form: F;
  reason: DraftReason;
}

/** Unsaved forms that survive a page change, such as the redirect after a create or a new login. */
export const draftsStore = {
  post: signal<Draft<PostForm> | null>(null),
  site: signal<Draft<SiteForm> | null>(null),
};
