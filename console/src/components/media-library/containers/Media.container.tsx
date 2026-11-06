import type { JSX } from "preact";
import { useMedia } from "../hooks/useMedia.hook";
import { MediaView } from "../ui/Media.view";

export const MediaContainer = (): JSX.Element => <MediaView {...useMedia()} />;
