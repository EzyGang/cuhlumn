import type { Kind } from "./api/Kind";
import type { Status } from "./api/Status";
import type { LoadState } from "./load";
import type { SelectItem } from "./select";
import type { BadgeTone } from "../components/shared/ui/Badge.view";

export type StatusFilter = "all" | Status;
export type KindFilter = "all" | Kind;

export interface PostFilters {
  status: StatusFilter;
  kind: KindFilter;
  query: string;
}

export interface PostRow {
  slug: string;
  title: string;
  status: Status;
  statusTone: BadgeTone;
  kind: Kind;
  updated: string;
  editHref: string;
  publicUrl: string | null;
}

export interface PostsModel {
  data: {
    rows: PostRow[];
    countText: string;
    emptyMessage: string;
    filters: PostFilters;
    statusItems: readonly SelectItem<StatusFilter>[];
    kindItems: readonly SelectItem<KindFilter>[];
    newHref: string;
  };
  status: { load: LoadState };
  actions: {
    setStatus: (status: StatusFilter) => void;
    setKind: (kind: KindFilter) => void;
    setQuery: (query: string) => void;
  };
}
