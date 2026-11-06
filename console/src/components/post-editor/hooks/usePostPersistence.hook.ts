import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useLocation } from "wouter-preact";
import { ApiError } from "../../../services/http";
import { deletePost, getPost, savePost } from "../../../services/posts.service";
import { draftsStore } from "../../../stores/drafts.store";
import type { Post } from "../../../types/api/Post";
import type { Status } from "../../../types/api/Status";
import type { LoadState } from "../../../types/load";
import type { Navigate } from "../../../types/navigation";
import type { PostForm } from "../../../types/postEditor";
import { postDraftKey } from "../../../utils/drafts";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { formToPostInput, postFieldPaths, postToForm } from "../../../utils/postForm";
import { reportApiError, reportFormError } from "../../../utils/reportApiError";
import { showToast } from "../../shared/ui/Toast.view";
import { usePostLoader } from "./usePostLoader.hook";

export interface PostPersistence {
  saved: Post | null;
  load: LoadState;
  saving: boolean;
  save: (status: Status) => void;
  remove: () => void;
}

interface PostTarget {
  host: string;
  slug: string | null;
  form: Signal<PostForm>;
  errors: Signal<FieldErrors>;
  saved: Signal<Post | null>;
  navigate: Navigate;
}

const savedMessage = (previous: Status | null, next: Status): string => {
  if (next === "draft") {
    return previous === "published" ? "Post unpublished" : "Draft saved";
  }
  return previous === "published" ? "Post saved" : "Post published";
};

/** Returns an error for the new slug, or null when no post of the site uses the slug yet. */
const checkNewSlug = async (host: string, slug: string): Promise<string | null> => {
  if (slug === "") {
    return "Enter a slug.";
  }
  try {
    await getPost(host, slug);
    return "A post with this slug exists already.";
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) {
      return null;
    }
    if (error instanceof ApiError && error.status === 400) {
      return error.message;
    }
    throw error;
  }
};

const postsHref = (host: string): string => `/sites/${encodeURIComponent(host)}/posts`;

/** Opens the saved new post, and hands over text that was typed while the request ran. */
const openCreated = (target: PostTarget, post: Post, sent: PostForm): void => {
  if (target.form.value !== sent) {
    const key = postDraftKey(target.host, post.slug);
    draftsStore.post.value = { key, form: target.form.value, reason: "redirect" };
  }
  target.navigate(`${postsHref(target.host)}/${encodeURIComponent(post.slug)}`, { replace: true });
};

const persist = async (target: PostTarget, status: Status): Promise<void> => {
  const { host, slug, form, errors, saved } = target;
  const sent = form.value;
  const name = slug ?? sent.slug.trim();
  const slugError = slug === null ? await checkNewSlug(host, name) : null;
  if (slugError !== null) {
    errors.value = { slug: slugError };
    return;
  }
  const result = await savePost(host, name, formToPostInput(sent, status));
  showToast("success", savedMessage(saved.value?.status ?? null, result.post.status));
  saved.value = result.post;
  errors.value = {};
  if (slug === null) {
    openCreated(target, result.post, sent);
  } else if (form.value === sent) {
    form.value = postToForm(result.post);
  }
};

const save = async (target: PostTarget, status: Status, saving: Signal<boolean>): Promise<void> => {
  saving.value = true;
  try {
    await persist(target, status);
  } catch (error) {
    const known = postFieldPaths(target.form.value);
    target.errors.value = reportFormError(error, known, "Cannot save the post");
  } finally {
    saving.value = false;
  }
};

const remove = async ({ host, slug, navigate }: PostTarget): Promise<void> => {
  if (slug === null) {
    return;
  }
  try {
    await deletePost(host, slug);
    showToast("success", "Post deleted", slug);
    navigate(postsHref(host), { replace: true });
  } catch (error) {
    reportApiError(error, "Cannot delete the post");
  }
};

export const usePostPersistence = (
  host: string,
  slug: string | null,
  form: Signal<PostForm>,
  errors: Signal<FieldErrors>,
): PostPersistence => {
  const [, navigate] = useLocation();
  const saved = useSignal<Post | null>(null);
  const saving = useSignal(false);
  const load = usePostLoader(host, slug, form, saved);
  const target: PostTarget = { host, slug, form, errors, saved, navigate };
  return {
    saved: saved.value,
    load,
    saving: saving.value,
    save: (status: Status): void => void save(target, status, saving),
    remove: (): void => void remove(target),
  };
};
