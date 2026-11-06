import type { JSX } from "preact";
import { ChipInput } from "../../shared/ui/ChipInput.view";
import { Section } from "../../shared/ui/Section.view";
import { TextField } from "../../shared/ui/TextField.view";
import { usePostEditorContext } from "../context/PostEditorContext";
import { IdentityFields } from "./IdentityFields.view";
import { ImageUrlFields } from "./ImageUrlFields.view";

export const DetailsSection = (): JSX.Element => {
  const { data, actions } = usePostEditorContext();
  return (
    <Section title="Details">
      <IdentityFields />
      <TextField
        label="Description"
        value={data.form.description}
        onValueChange={actions.text.description}
        error={data.errors["description"]}
        multiline={2}
      />
      <ChipInput
        label="Tags"
        model={data.tags}
        error={data.tagError}
        placeholder="Type a tag and press Enter"
        description="At most 20 tags."
      />
      <ImageUrlFields />
    </Section>
  );
};
