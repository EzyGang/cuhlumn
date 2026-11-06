import type { Signal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { useLoad } from "../../../hooks/useLoad.hook";
import { getPost } from "../../../services/posts.service";
import { draftsStore } from "../../../stores/drafts.store";
import { sessionStore } from "../../../stores/session.store";
import type { Post } from "../../../types/api/Post";
import type { LoadState } from "../../../types/load";
import type { PostForm } from "../../../types/postEditor";
import { formChanged, postDraftKey, takeDraft } from "../../../utils/drafts";
import { emptyPostForm, postToForm } from "../../../utils/postForm";
import { showToast } from "../../shared/ui/Toast.view";

/** Returns the unsaved form of this post from before a redirect or a new login. */
const restoreDraft = (key: string): PostForm | null => {
  const draft = takeDraft(draftsStore.post, key);
  if (draft?.reason === "session") {
    showToast("success", "Unsaved changes restored");
  }
  return draft?.form ?? null;
};

/** Keeps the unsaved form when the session ends, so a new login can restore it. */
const useSessionDraft = (
  key: string,
  creating: boolean,
  form: Signal<PostForm>,
  saved: Signal<Post | null>,
): void => {
  useEffect((): (() => void) => {
    const restored = creating ? restoreDraft(key) : null;
    if (restored !== null) {
      form.value = restored;
    }
    return (): void => {
      const baseline = saved.value === null ? emptyPostForm() : postToForm(saved.value);
      if (sessionStore.status.value === "anonymous" && formChanged(form.value, baseline)) {
        draftsStore.post.value = { key, form: form.value, reason: "session" };
      }
    };
  }, [key, creating, form, saved]);
};

export const usePostLoader = (
  host: string,
  slug: string | null,
  form: Signal<PostForm>,
  saved: Signal<Post | null>,
): LoadState => {
  const key = postDraftKey(host, slug);
  const load = async (name: string): Promise<Post> => {
    const post = await getPost(host, name);
    saved.value = post;
    form.value = restoreDraft(key) ?? postToForm(post);
    return post;
  };
  const loaded = useLoad(key, slug === null ? null : (): Promise<Post> => load(slug));
  useSessionDraft(key, slug === null, form, saved);
  return { status: loaded.status, error: loaded.error, retry: loaded.retry };
};
