import type { JSX } from "preact";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { ImageField } from "./ImageField.view";

export const ImageFields = (): JSX.Element => {
  const { data } = useSiteSettingsContext();
  return (
    <>
      <ImageField field={data.images.logo} picker={data.picker} />
      <ImageField field={data.images.favicon} picker={data.picker} />
    </>
  );
};
