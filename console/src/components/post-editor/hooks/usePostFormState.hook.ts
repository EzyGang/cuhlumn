import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useChipInput } from "../../../hooks/useChipInput.hook";
import type { Kind } from "../../../types/api/Kind";
import type { PostForm, PostTextKey } from "../../../types/postEditor";
import { TAG_RULES } from "../../../utils/chips";
import { bindKeys } from "../../../utils/bindKeys";
import { firstItemError, withoutFieldError } from "../../../utils/fieldErrors";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { suggestSlug } from "../../../utils/slug";
import type { PostEditorContextValue } from "../context/PostEditorContext";

export type PostFormState = Pick<PostEditorContextValue["data"], "tags" | "tagError"> & {
  actions: Pick<PostEditorContextValue["actions"], "text" | "setKind">;
};

const TEXT_KEYS: readonly PostTextKey[] = [
  "slug",
  "title",
  "description",
  "body",
  "cover",
  "og_image",
];

type Update = (patch: Partial<PostForm>, path: string) => void;

const useTagInput = (
  form: Signal<PostForm>,
  errors: Signal<FieldErrors>,
  update: Update,
): Omit<PostFormState, "actions"> => {
  const [tagIndex, tagError] = firstItemError(errors.value, "tags");
  const setTags = (values: string[]): void => update({ tags: values }, "tags");
  return { tags: useChipInput(form.value.tags, setTags, TAG_RULES, tagIndex), tagError };
};

export const usePostFormState = (
  form: Signal<PostForm>,
  errors: Signal<FieldErrors>,
  creating: boolean,
): PostFormState => {
  const slugTouched = useSignal(false);
  const update: Update = (patch, path) => {
    form.value = { ...form.value, ...patch };
    errors.value = withoutFieldError(errors.value, path);
  };
  const setText = (key: PostTextKey, value: string): void => {
    slugTouched.value = slugTouched.value || key === "slug";
    const follow = creating && key === "title" && !slugTouched.value;
    update(follow ? { title: value, slug: suggestSlug(value) } : { [key]: value }, key);
  };
  return {
    ...useTagInput(form, errors, update),
    actions: {
      text: bindKeys(TEXT_KEYS, setText),
      setKind: (kind: Kind): void => update({ kind }, "kind"),
    },
  };
};
