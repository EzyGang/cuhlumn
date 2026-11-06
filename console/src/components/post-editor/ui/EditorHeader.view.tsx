import type { JSX } from "preact";
import { Badge } from "../../shared/ui/Badge.view";
import { ExternalLink } from "../../shared/ui/ExternalLink.view";
import { PageHeader } from "../../shared/ui/PageHeader.view";
import { usePostEditorContext } from "../context/PostEditorContext";
import { EditorActions } from "./EditorActions.view";

export const EditorHeader = (): JSX.Element => {
  const { data } = usePostEditorContext();
  return (
    <PageHeader
      eyebrow={
        <>
          {data.mode === "create" ? "New post" : "Edit post"}
          {data.savedStatus === null ? null : (
            <Badge tone={data.statusTone}>{data.savedStatus}</Badge>
          )}
        </>
      }
      title={data.form.title || "Untitled"}
      below={
        data.publicUrl === null ? null : (
          <ExternalLink href={data.publicUrl}>{data.publicUrl}</ExternalLink>
        )
      }
    >
      <EditorActions />
    </PageHeader>
  );
};
