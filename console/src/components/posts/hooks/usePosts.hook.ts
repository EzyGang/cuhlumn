import { useSignal } from "@preact/signals";
import { useParams } from "wouter-preact";
import { useLoad } from "../../../hooks/useLoad.hook";
import { listPosts } from "../../../services/posts.service";
import type { Post } from "../../../types/api/Post";
import type {
  KindFilter,
  PostFilters,
  PostRow,
  PostsModel,
  StatusFilter,
} from "../../../types/posts";
import type { SelectItem } from "../../../types/select";
import { formatDateTime } from "../../../utils/format";
import { statusTone } from "../../../utils/status";

const STATUS_ITEMS: readonly SelectItem<StatusFilter>[] = [
  { value: "all", label: "All statuses" },
  { value: "draft", label: "Drafts" },
  { value: "published", label: "Published" },
];

const KIND_ITEMS: readonly SelectItem<KindFilter>[] = [
  { value: "all", label: "Posts and pages" },
  { value: "post", label: "Posts" },
  { value: "page", label: "Pages" },
];

const toRow = (host: string, post: Post): PostRow => ({
  slug: post.slug,
  title: post.title,
  status: post.status,
  statusTone: statusTone(post.status),
  kind: post.kind,
  updated: formatDateTime(post.updated_at),
  editHref: `/sites/${encodeURIComponent(host)}/posts/${encodeURIComponent(post.slug)}`,
  publicUrl: post.status === "published" ? post.url : null,
});

const matches = (post: Post, filters: PostFilters): boolean => {
  const text = filters.query.trim().toLowerCase();
  return (
    (filters.status === "all" || post.status === filters.status) &&
    (filters.kind === "all" || post.kind === filters.kind) &&
    (text === "" || post.title.toLowerCase().includes(text) || post.slug.includes(text))
  );
};

export const usePosts = (): PostsModel => {
  const host = useParams<{ host: string }>().host;
  const loaded = useLoad(host, (): Promise<Post[]> => listPosts(host));
  const posts = loaded.value;
  const filters = useSignal<PostFilters>({ status: "all", kind: "all", query: "" });
  const setFilter = (patch: Partial<PostFilters>): void => {
    filters.value = { ...filters.value, ...patch };
  };
  const rows = (posts ?? [])
    .filter((post) => matches(post, filters.value))
    .map((post) => toRow(host, post));
  return {
    data: {
      rows,
      countText: `${rows.length} of ${posts?.length ?? 0}`,
      emptyMessage: posts?.length === 0 ? "There are no posts yet." : "No posts match the filters.",
      filters: filters.value,
      statusItems: STATUS_ITEMS,
      kindItems: KIND_ITEMS,
      newHref: `/sites/${encodeURIComponent(host)}/new-post`,
    },
    status: { load: { status: loaded.status, error: loaded.error, retry: loaded.retry } },
    actions: {
      setStatus: (status: StatusFilter): void => setFilter({ status }),
      setKind: (kind: KindFilter): void => setFilter({ kind }),
      setQuery: (query: string): void => setFilter({ query }),
    },
  };
};
