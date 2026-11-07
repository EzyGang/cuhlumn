import type { TocEntry } from "../types/api/TocEntry";
import type { PreviewModel, PreviewState, TocItem } from "../types/postEditor";

const INDENT: Record<number, string> = { 2: "pl-0", 3: "pl-3", 4: "pl-6", 5: "pl-9", 6: "pl-12" };

const MESSAGES: Record<Exclude<PreviewState["kind"], "ready" | "failed">, string> = {
  empty: "The preview shows here when the post has text.",
  loading: "Rendering the preview",
  unavailable: "Preview unavailable: the server does not render previews yet.",
};

/**
 * Wraps rendered post HTML and the site stylesheet in a full document for a sandboxed iframe.
 * The markup repeats the post page structure, so the `.column` and `.post-body` rules of the site apply.
 * The document has no `data-theme`, so the preview follows the system theme.
 */
export const previewDocument = (html: string, css: string): string =>
  `<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><style>${css.replaceAll("</style", "<\\/style")}</style></head><body><main class="column"><article><div class="post-body">${html}</div></article></main></body></html>`;

const tocItem = (entry: TocEntry): TocItem => ({
  id: entry.id,
  label: entry.label,
  indent: INDENT[entry.level] ?? "pl-12",
});

export const toPreviewModel = (state: PreviewState): PreviewModel => {
  switch (state.kind) {
    case "ready":
      return {
        message: null,
        document: previewDocument(state.html, state.css),
        toc: state.toc.map(tocItem),
        wordCount: state.wordCount === 1 ? "1 word" : `${state.wordCount} words`,
      };
    case "failed":
      return {
        message: `Preview failed: ${state.message}`,
        document: null,
        toc: [],
        wordCount: null,
      };
    default:
      return { message: MESSAGES[state.kind], document: null, toc: [], wordCount: null };
  }
};
