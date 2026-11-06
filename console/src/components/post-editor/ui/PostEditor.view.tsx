import type { JSX } from "preact";
import { LoadGate } from "../../shared/ui/LoadGate.view";
import { usePostEditorContext } from "../context/PostEditorContext";
import { ContentSection } from "./ContentSection.view";
import { DetailsSection } from "./DetailsSection.view";
import { EditorHeader } from "./EditorHeader.view";

export const PostEditorView = (): JSX.Element => {
  const { status } = usePostEditorContext();
  return (
    <LoadGate
      state={status.load}
      loadingText="Loading the post"
      missingText="This post does not exist."
    >
      <div class="flex flex-col gap-6">
        <EditorHeader />
        <DetailsSection />
        <ContentSection />
      </div>
    </LoadGate>
  );
};
