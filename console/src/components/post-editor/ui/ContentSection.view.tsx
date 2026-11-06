import type { JSX } from "preact";
import { Field } from "../../shared/ui/Field.view";
import { Textarea } from "../../shared/ui/Textarea.view";
import { usePostEditorContext } from "../context/PostEditorContext";
import { PreviewPane } from "./PreviewPane.view";

export const ContentSection = (): JSX.Element => {
  const { data, actions } = usePostEditorContext();
  return (
    <div class="grid gap-6 lg:grid-cols-2">
      <Field label="Markdown" error={data.errors["body"]}>
        <Textarea
          value={data.form.body}
          onValueChange={actions.text.body}
          rows={28}
          placeholder="Write the post in Markdown."
          mono
          class="min-h-[28rem]"
        />
      </Field>
      <PreviewPane preview={data.preview} />
    </div>
  );
};
