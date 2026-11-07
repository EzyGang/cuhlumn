import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import { useChipInput } from "../../../hooks/useChipInput.hook";
import { useSites } from "../../../hooks/useSites.hook";
import type { Kind } from "../../../types/api/Kind";
import type { CtaChoice, PostForm, PostTextKey } from "../../../types/postEditor";
import type { SelectItem } from "../../../types/select";
import { TAG_RULES } from "../../../utils/chips";
import { bindKeys } from "../../../utils/bindKeys";
import { firstItemError, withoutFieldError } from "../../../utils/fieldErrors";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { ctaItems } from "../../../utils/postForm";
import { suggestSlug } from "../../../utils/slug";
import type { PostEditorContextValue } from "../context/PostEditorContext";

export type PostFormState = Pick<
  PostEditorContextValue["data"],
  "tags" | "tagError" | "ctaChoices"
> & {
  actions: Pick<PostEditorContextValue["actions"], "text" | "setKind" | "setShowCta">;
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
): Pick<PostFormState, "tags" | "tagError"> => {
  const [tagIndex, tagError] = firstItemError(errors.value, "tags");
  const setTags = (values: string[]): void => update({ tags: values }, "tags");
  return { tags: useChipInput(form.value.tags, setTags, TAG_RULES, tagIndex), tagError };
};

/** Returns the call to action choices, with the default of the site in the first label. */
const useCtaChoices = (host: string): readonly SelectItem<CtaChoice>[] => {
  const site = useSites().sites?.find((item) => item.host === host);
  return ctaItems(site?.show_cta ?? null);
};

export const usePostFormState = (
  host: string,
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
    ctaChoices: useCtaChoices(host),
    actions: {
      text: bindKeys(TEXT_KEYS, setText),
      setKind: (kind: Kind): void => update({ kind }, "kind"),
      setShowCta: (show_cta: CtaChoice): void => update({ show_cta }, "show_cta"),
    },
  };
};
