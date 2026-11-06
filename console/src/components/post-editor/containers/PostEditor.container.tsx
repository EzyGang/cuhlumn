import type { JSX } from "preact";
import { PostEditorProvider } from "../context/PostEditorContext";
import { usePostEditor } from "../hooks/usePostEditor.hook";
import { PostEditorView } from "../ui/PostEditor.view";

export const PostEditorContainer = (): JSX.Element => (
  <PostEditorProvider value={usePostEditor()}>
    <PostEditorView />
  </PostEditorProvider>
);
