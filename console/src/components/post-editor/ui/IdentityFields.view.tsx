import type { JSX } from "preact";
import { Select } from "../../shared/ui/Select.view";
import { TextField } from "../../shared/ui/TextField.view";
import { usePostEditorContext } from "../context/PostEditorContext";

export const IdentityFields = (): JSX.Element => {
  const { data, actions } = usePostEditorContext();
  const { form, errors } = data;
  return (
    <div class="grid gap-5 md:grid-cols-[2fr_1fr_auto]">
      <TextField
        label="Title"
        value={form.title}
        onValueChange={actions.text.title}
        error={errors["title"]}
      />
      <TextField
        label="Slug"
        description={
          data.mode === "create" ? "Suggested from the title." : "The slug cannot change."
        }
        value={form.slug}
        onValueChange={actions.text.slug}
        error={errors["slug"]}
        disabled={data.mode === "edit"}
        mono
      />
      <Select label="Kind" value={form.kind} items={data.kinds} onValueChange={actions.setKind} />
    </div>
  );
};
