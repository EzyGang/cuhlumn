import type { Signal } from "@preact/signals";
import type { Draft } from "../stores/drafts.store";

/** Removes and returns the stored draft when it belongs to `key`. */
export const takeDraft = <F>(store: Signal<Draft<F> | null>, key: string): Draft<F> | null => {
  const draft = store.value;
  if (draft === null || draft.key !== key) {
    return null;
  }
  store.value = null;
  return draft;
};

/** Returns true when two forms hold different values. */
export const formChanged = <F>(form: F, baseline: F): boolean =>
  JSON.stringify(form) !== JSON.stringify(baseline);

export const postDraftKey = (host: string, slug: string | null): string => `${host}/${slug ?? ""}`;
