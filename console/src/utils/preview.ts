import type { TocEntry } from "../types/api/TocEntry";
import type { PreviewModel, PreviewState, TocItem } from "../types/postEditor";

const INDENT: Record<number, string> = { 2: "pl-0", 3: "pl-3", 4: "pl-6", 5: "pl-9", 6: "pl-12" };

const MESSAGES: Record<Exclude<PreviewState["kind"], "ready" | "failed">, string> = {
  empty: "The preview shows here when the post has text.",
  loading: "Rendering the preview",
  unavailable: "Preview unavailable: the server does not render previews yet.",
};

const PREVIEW_CSS = `
:root { color-scheme: dark; }
body { margin: 0; padding: 20px; background: oklch(14% 0 0); color: oklch(100% 0 0 / 0.8);
  font: 16px/1.65 Inter, ui-sans-serif, system-ui, sans-serif; overflow-wrap: break-word; }
h1, h2, h3, h4, h5, h6 { color: oklch(100% 0 0); line-height: 1.25; font-weight: 600; margin: 1.4em 0 0.5em; }
h1 { font-size: 1.875rem; } h2 { font-size: 1.5rem; } h3 { font-size: 1.25rem; }
body > :first-child { margin-top: 0; }
p, ul, ol, blockquote, pre, table { margin: 0 0 1em; }
a { color: oklch(100% 0 0); }
ul, ol { padding-left: 1.5em; }
blockquote { border-left: 2px solid oklch(100% 0 0 / 0.25); margin-left: 0; padding-left: 1em; color: oklch(65% 0 0); }
code { font-family: "JetBrains Mono", ui-monospace, monospace; font-size: 0.875em; }
:not(pre) > code { background: oklch(22% 0 0); padding: 0.1em 0.3em; }
pre { overflow-x: auto; background: oklch(0% 0 0); border: 1px solid oklch(100% 0 0 / 0.1); padding: 1em; }
img { max-width: 100%; height: auto; }
hr { border: 0; border-top: 1px solid oklch(100% 0 0 / 0.1); }
table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
th, td { border: 1px solid oklch(100% 0 0 / 0.1); padding: 0.5em 0.75em; text-align: left; }
.heading-link { margin-left: 0.4em; color: oklch(65% 0 0); font-weight: 400; text-decoration: none; opacity: 0; }
h2:hover > .heading-link, h3:hover > .heading-link { opacity: 1; }
dl.term-list { margin: 0 0 1em; }
dl.term-list > div { display: grid; grid-template-columns: minmax(6rem, 1fr) 2fr; gap: 0.75em;
  padding: 0.5em 0; border-top: 1px solid oklch(100% 0 0 / 0.1); }
dl.term-list dt { color: oklch(100% 0 0); font-weight: 600; }
dl.term-list dd { margin: 0; }
`;

/** Wraps rendered post HTML in a full document for a sandboxed iframe. */
export const previewDocument = (html: string): string =>
  `<!doctype html><html><head><meta charset="utf-8"><style>${PREVIEW_CSS}</style></head><body>${html}</body></html>`;

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
        document: previewDocument(state.html),
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
