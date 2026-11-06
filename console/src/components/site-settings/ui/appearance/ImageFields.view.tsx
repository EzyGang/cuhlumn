import type { JSX } from "preact";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { ImageField } from "./ImageField.view";

export const ImageFields = (): JSX.Element => {
  const { data } = useSiteSettingsContext();
  return (
    <>
      {data.imageFields.map((field) => (
        <ImageField key={field.name} field={field} picker={data.picker} />
      ))}
    </>
  );
};
