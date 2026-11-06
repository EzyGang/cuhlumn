import type { Post } from "../types/api/Post";
import type { PostInput } from "../types/api/PostInput";
import type { PostSaved } from "../types/api/PostSaved";
import type { Preview } from "../types/api/Preview";
import type { PreviewInput } from "../types/api/PreviewInput";
import { requestEmpty, requestJson, segment } from "./http";

const postsPath = (host: string): string => `/api/sites/${segment(host)}/posts`;

export const listPosts = (host: string): Promise<Post[]> =>
  requestJson<Post[]>("GET", postsPath(host));

export const getPost = (host: string, slug: string): Promise<Post> =>
  requestJson<Post>("GET", `${postsPath(host)}/${segment(slug)}`);

export const savePost = (host: string, slug: string, input: PostInput): Promise<PostSaved> =>
  requestJson<PostSaved>("PUT", `${postsPath(host)}/${segment(slug)}`, { json: input });

export const deletePost = (host: string, slug: string): Promise<void> =>
  requestEmpty("DELETE", `${postsPath(host)}/${segment(slug)}`);

export const previewMarkdown = (host: string, input: PreviewInput): Promise<Preview> =>
  requestJson<Preview>("POST", `/api/sites/${segment(host)}/preview`, { json: input });
