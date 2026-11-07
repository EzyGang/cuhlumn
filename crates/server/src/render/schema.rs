//! JSON-LD structured data ([schema.org](https://schema.org)) for search engines and AI assistants.

use maud::{Markup, PreEscaped, html};
use serde_json::{Value, json};

use crate::render::{
    blog::{Blog, Entry},
    iso, social,
};

/// The number of posts that the home page data lists.
const HOME_POSTS: usize = 30;

/// Returns the `Blog` data of the home page, or `WebSite` data when a page is the home page.
#[must_use]
pub fn home(blog: &Blog, description: &str) -> Value {
    let site = &blog.site;
    let kind = match blog.home_page() {
        Some(_) => "WebSite",
        None => "Blog",
    };
    let posts: Vec<Value> = blog
        .articles()
        .take(HOME_POSTS)
        .map(|entry| {
            json!({"@type": "BlogPosting", "headline": entry.post.title, "url": entry.post.url,
                   "datePublished": entry.post.published_at.map(iso)})
        })
        .collect();
    json!({
        "@context": "https://schema.org", "@type": kind, "name": site.title, "url": blog.root_url(),
        "description": description, "inLanguage": site.lang, "publisher": publisher(blog), "blogPost": posts,
    })
}

/// Returns the `BlogPosting` or `WebPage` data of an entry with its breadcrumbs.
#[must_use]
pub fn article(blog: &Blog, entry: &Entry, description: &str, image: &str) -> Value {
    let (site, post) = (&blog.site, &entry.post);
    let (kind, parent) = match entry.is_page() {
        true => ("WebPage", "WebSite"),
        false => ("BlogPosting", "Blog"),
    };
    let mut thing = json!({
        "@type": kind, "headline": post.title, "name": post.title, "description": description,
        "url": post.url, "mainEntityOfPage": post.url, "image": [image], "inLanguage": site.lang,
        "author": author(blog), "publisher": publisher(blog), "dateModified": iso(post.updated_at),
        "wordCount": entry.doc().words, "keywords": post.tags,
        "isPartOf": {"@type": parent, "name": site.title, "url": blog.root_url()},
    });
    if let Some(time) = post.published_at {
        thing["datePublished"] = json!(iso(time));
    }
    let crumbs = json!({"@type": "BreadcrumbList", "itemListElement": [
        {"@type": "ListItem", "position": 1, "name": site.title, "item": blog.root_url()},
        {"@type": "ListItem", "position": 2, "name": post.title, "item": post.url}]});
    json!({"@context": "https://schema.org", "@graph": [thing, crumbs]})
}

/// Returns the `CollectionPage` data of a tag page.
#[must_use]
pub fn tag(name: &str, url: &str) -> Value {
    json!({"@context": "https://schema.org", "@type": "CollectionPage", "name": format!("#{name}"), "url": url})
}

/// Returns a JSON-LD script element.
///
/// Every `<` becomes the JSON escape `\u003c`, so text can neither close the element nor open a comment.
#[must_use]
pub fn script(data: &Value) -> Markup {
    let text = data.to_string().replace('<', "\\u003c");
    html! { script type="application/ld+json" { (PreEscaped(text)) } }
}

fn publisher(blog: &Blog) -> Value {
    let brand = &blog.site.brand;
    let logo = match &brand.logo {
        Some(logo) => blog.absolute(logo),
        None => format!("{}/favicon.svg", blog.origin),
    };
    let mut publisher = json!({"@type": "Organization", "name": blog.site.title, "url": blog.root_url(),
           "logo": {"@type": "ImageObject", "url": logo}});
    let profiles = social::profiles(&brand.social);
    if !profiles.is_empty() {
        publisher["sameAs"] = json!(profiles);
    }
    publisher
}

fn author(blog: &Blog) -> Value {
    let site = &blog.site;
    if site.author.trim().is_empty() {
        return publisher(blog);
    }
    match &site.author_url {
        Some(url) => json!({"@type": "Person", "name": site.author, "url": url}),
        None => json!({"@type": "Person", "name": site.author}),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::render::schema::script;

    #[test]
    fn script_text_has_no_less_than_sign() {
        let data = json!({"name": "<!-- <script> </script>"});
        let html = script(&data).into_string();
        let inner = html
            .strip_prefix(r#"<script type="application/ld+json">"#)
            .and_then(|rest| rest.strip_suffix("</script>"))
            .unwrap();
        assert!(!inner.contains('<'), "{inner}");
        let parsed: Value = serde_json::from_str(inner).unwrap();
        assert_eq!(parsed, data);
    }
}
