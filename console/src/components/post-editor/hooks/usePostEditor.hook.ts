import { useSignal } from "@preact/signals";
import { useParams } from "wouter-preact";
import type { Kind } from "../../../types/api/Kind";
import type { PostForm } from "../../../types/postEditor";
import type { SelectItem } from "../../../types/select";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { emptyPostForm } from "../../../utils/postForm";
import { statusTone } from "../../../utils/status";
import type { PostEditorContextValue } from "../context/PostEditorContext";
import { usePostFormState } from "./usePostFormState.hook";
import type { PostFormState } from "./usePostFormState.hook";
import { usePostPersistence } from "./usePostPersistence.hook";
import type { PostPersistence } from "./usePostPersistence.hook";
import { usePreview } from "./usePreview.hook";

const KINDS: readonly SelectItem<Kind>[] = [
  { value: "post", label: "Post" },
  { value: "page", label: "Page" },
];

const editorActions = (
  state: PostFormState,
  persistence: PostPersistence,
): PostEditorContextValue["actions"] => ({
  ...state.actions,
  primary: (): void => persistence.save("published"),
  secondary: (): void => persistence.save("draft"),
  remove: persistence.remove,
});

export const usePostEditor = (): PostEditorContextValue => {
  const params = useParams<{ host: string; slug?: string }>();
  const slug = params.slug ?? null;
  const form = useSignal<PostForm>(emptyPostForm());
  const errors = useSignal<FieldErrors>({});
  const state = usePostFormState(params.host, form, errors, slug === null);
  const persistence = usePostPersistence(params.host, slug, form, errors);
  const preview = usePreview(params.host, form);
  const saved = persistence.saved;
  const published = saved?.status === "published";
  return {
    data: {
      mode: slug === null ? "create" : "edit",
      form: form.value,
      errors: errors.value,
      tags: state.tags,
      tagError: state.tagError,
      kinds: KINDS,
      ctaChoices: state.ctaChoices,
      savedStatus: saved?.status ?? null,
      statusTone: statusTone(saved?.status ?? "draft"),
      publicUrl: saved?.status === "published" ? saved.url : null,
      primaryLabel: published ? "Save" : "Publish",
      secondaryLabel: published ? "Unpublish" : "Save draft",
      preview,
    },
    status: { load: persistence.load, saving: persistence.saving },
    actions: editorActions(state, persistence),
  };
};
