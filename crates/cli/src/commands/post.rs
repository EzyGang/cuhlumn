//! `cuhlumn post` commands.

use std::path::Path;

use anyhow::{Context as _, bail};
use cuhlumn_api::{
    post::{Kind, Post, PostInput, Status, slug::Slug},
    site::host::Host,
    wire::{PostSaved, PreviewInput},
};
use serde::Serialize;

use crate::{
    cli::PostCommand,
    client::Client,
    commands::{Context, assigned, confirm, print_json, read_text, write_stdout},
    front_matter::{self, PostFile},
};

/// One entry of `post list`, without the body.
#[derive(Debug, Serialize)]
struct Summary {
    slug: Slug,
    title: String,
    kind: Kind,
    status: Status,
    published_at: Option<i64>,
    updated_at: i64,
    url: String,
}

pub(super) fn run(context: &Context, command: PostCommand) -> anyhow::Result<()> {
    let client = &context.client;
    let host = context.site()?;
    match command {
        PostCommand::List { status, kind } => {
            let summaries: Vec<Summary> = client
                .posts(host)?
                .into_iter()
                .filter(|post| status.is_none_or(|status| post.status == status))
                .filter(|post| kind.is_none_or(|kind| post.kind == kind))
                .map(Summary::from)
                .collect();
            print_json(&summaries)
        }
        PostCommand::Get { slug, markdown } => {
            let post = client.post(host, &slug)?;
            if markdown {
                return write_stdout(&front_matter::to_markdown(&post));
            }
            print_json(&post)
        }
        PostCommand::Push {
            file,
            publish,
            draft,
        } => print_json(&push(client, host, &file, publish, draft)?),
        PostCommand::Set { slug, assignments } => {
            let current = PostInput::from(client.post(host, &slug)?);
            let input: PostInput = assigned(&current, &assignments)?;
            print_json(&client.put_post(host, &slug, &input)?)
        }
        PostCommand::Publish { slug } => {
            print_json(&set_status(client, host, &slug, Status::Published)?)
        }
        PostCommand::Unpublish { slug } => {
            print_json(&set_status(client, host, &slug, Status::Draft)?)
        }
        PostCommand::Delete { slug, yes } => {
            confirm(yes, &format!("the post {slug}"))?;
            Ok(client.delete_post(host, &slug)?)
        }
        PostCommand::Preview { file } => {
            let text = read_text(&file)?;
            let body = front_matter::body(&text)
                .with_context(|| format!("cannot read the post in {}", file.display()))?;
            let input = PreviewInput { body };
            print_json(&client.preview(host, &input)?)
        }
    }
}

impl From<Post> for Summary {
    fn from(post: Post) -> Self {
        Self {
            slug: post.slug,
            title: post.title,
            kind: post.kind,
            status: post.status,
            published_at: post.published_at,
            updated_at: post.updated_at,
            url: post.url,
        }
    }
}

/// Saves the post in a Markdown file.
fn push(
    client: &Client,
    host: &Host,
    file: &Path,
    publish: bool,
    draft: bool,
) -> anyhow::Result<PostSaved> {
    let text = read_text(file)?;
    let PostFile { slug, mut input } = PostFile::parse(&text)
        .with_context(|| format!("cannot read the post in {}", file.display()))?;
    let slug = match slug {
        Some(slug) => slug,
        None => slug_from_file(file)?,
    };
    if publish {
        input.status = Status::Published;
    }
    if draft {
        input.status = Status::Draft;
    }
    if input.status == Status::Draft
        && !draft
        && let Some(stored) = client.find_post(host, &slug)?
        && stored.status == Status::Published
    {
        bail!(
            "{slug} is published, and {} would make it a draft: add `status: published` to the front matter, \
             pass --publish to keep it public, or pass --draft to unpublish it",
            file.display()
        );
    }
    Ok(client.put_post(host, &slug, &input)?)
}

/// Returns the slug that the file name gives, such as `sqlite-backups` for `sqlite-backups.md`.
fn slug_from_file(file: &Path) -> anyhow::Result<Slug> {
    let stem = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("the file has no name to use as the slug: add a slug field to the front matter")?;
    Slug::new(stem).with_context(|| {
        format!("the file name {stem:?} is not a valid slug: add a slug field to the front matter")
    })
}

/// Changes only the status of a post. A post that has the status already stays unchanged.
fn set_status(
    client: &Client,
    host: &Host,
    slug: &Slug,
    status: Status,
) -> anyhow::Result<PostSaved> {
    let post = client.post(host, slug)?;
    if post.status == status {
        return Ok(PostSaved {
            url: post.url.clone(),
            post,
        });
    }
    let mut input = PostInput::from(post);
    input.status = status;
    Ok(client.put_post(host, slug, &input)?)
}
