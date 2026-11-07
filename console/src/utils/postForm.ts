import type { Post } from "../types/api/Post";
import type { PostInput } from "../types/api/PostInput";
import type { Status } from "../types/api/Status";
import type { CtaChoice, PostForm } from "../types/postEditor";
import type { SelectItem } from "../types/select";
import { optional } from "./siteForm";

export const emptyPostForm = (): PostForm => ({
  slug: "",
  title: "",
  description: "",
  body: "",
  tags: [],
  kind: "post",
  cover: "",
  og_image: "",
  show_cta: "site",
});

/** Returns the form choice for the API value: null follows the site default. */
export const ctaChoice = (value: boolean | null): CtaChoice => {
  if (value === null) {
    return "site";
  }
  return value ? "show" : "hide";
};

/** Returns the API value for the form choice. */
export const ctaValue = (choice: CtaChoice): boolean | null => {
  if (choice === "site") {
    return null;
  }
  return choice === "show";
};

/** Lists the call to action choices. The first label tells the site default when it is known. */
export const ctaItems = (siteShows: boolean | null): SelectItem<CtaChoice>[] => {
  const site = siteShows === null ? "" : siteShows ? " (shown)" : " (hidden)";
  return [
    { value: "site", label: `Site default${site}` },
    { value: "show", label: "Show" },
    { value: "hide", label: "Hide" },
  ];
};

export const postToForm = (post: Post): PostForm => ({
  slug: post.slug,
  title: post.title,
  description: post.description,
  body: post.body,
  tags: [...post.tags],
  kind: post.kind,
  cover: post.cover ?? "",
  og_image: post.og_image ?? "",
  show_cta: ctaChoice(post.show_cta),
});

export const formToPostInput = (form: PostForm, status: Status): PostInput => ({
  title: form.title,
  description: form.description,
  body: form.body,
  tags: form.tags,
  kind: form.kind,
  status,
  cover: optional(form.cover),
  og_image: optional(form.og_image),
  show_cta: ctaValue(form.show_cta),
  published_at: null,
});

/** Lists the API field paths that the post form can show an error for. */
export const postFieldPaths = (form: PostForm): string[] => [
  "slug",
  "title",
  "description",
  "body",
  "tags",
  ...form.tags.map((_, index) => `tags.${index}`),
  "kind",
  "cover",
  "og_image",
  "show_cta",
];
