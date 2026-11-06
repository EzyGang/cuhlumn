import type { JSX } from "preact";
import type { PreviewModel } from "../../../types/postEditor";
import { TocView } from "./Toc.view";

interface PreviewPaneProps {
  preview: PreviewModel;
}

export const PreviewPane = ({ preview }: PreviewPaneProps): JSX.Element => (
  <section aria-label="Preview" class="flex min-w-0 flex-col gap-1.5">
    <h2 class="text-xs font-medium tracking-wide text-secondary uppercase">Preview</h2>
    <div class="flex min-h-[28rem] flex-1 flex-col gap-4 border border-border bg-surface p-5">
      {preview.message === null ? null : (
        <p class="text-sm text-muted" role="status">
          {preview.message}
        </p>
      )}
      {preview.wordCount === null ? null : <p class="eyebrow">{preview.wordCount}</p>}
      <TocView items={preview.toc} />
      {preview.document === null ? null : (
        <iframe
          title="Rendered post"
          sandbox=""
          srcdoc={preview.document}
          class="min-h-[36rem] w-full flex-1 border border-border bg-surface"
        />
      )}
    </div>
  </section>
);
