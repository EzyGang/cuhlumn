import type { JSX } from "preact";
import { usePosts } from "../hooks/usePosts.hook";
import { PostsView } from "../ui/Posts.view";

export const PostsContainer = (): JSX.Element => <PostsView {...usePosts()} />;
