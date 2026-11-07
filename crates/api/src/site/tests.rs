use serde_json::{Value, json};

use crate::site::{SiteInput, brand::Brand, theme::Font};

fn parse(value: Value) -> Result<SiteInput, serde_json::Error> {
    serde_json::from_value(value)
}

#[test]
fn omitted_fields_take_defaults() {
    let input = parse(json!({"title": "  Harbor Notes "})).unwrap();
    assert_eq!(input.title, "Harbor Notes");
    assert_eq!(input.lang.as_str(), "en");
    assert_eq!(input.brand, Brand::default());
    assert!(input.brand.credit);
    assert_eq!(input.brand.accent.as_str(), "#2563eb");
    assert_eq!(input.brand.font, Font::Serif);
    assert_eq!(input.indexnow_key, None);
}

#[test]
fn title_is_required_and_limited() {
    assert!(parse(json!({})).is_err());
    assert!(parse(json!({"title": "   "})).is_err());
    assert!(parse(json!({"title": "a".repeat(201)})).is_err());
    assert!(parse(json!({"title": "a".repeat(200)})).is_ok());
}

#[test]
fn control_characters_are_rejected() {
    for field in ["title", "description", "author", "project_name", "cta"] {
        let mut site = json!({"title": "x"});
        site[field] = json!("bad\u{1}text");
        assert!(parse(site).is_err(), "{field}");
    }
    let footer = json!({"title": "x", "brand": {"footer": "a\u{85}b"}});
    assert!(parse(footer).is_err());
    let label = json!({"title": "x", "brand": {"links": [{"label": "a\u{7f}", "url": "/a"}]}});
    assert!(parse(label).is_err());
    assert!(parse(json!({"title": "x", "description": "one\ntwo\tthree"})).is_ok());
}

#[test]
fn aliases_lose_repetitions() {
    let input =
        parse(json!({"title": "x", "aliases": ["WWW.a.com", "www.a.com", "b.com"]})).unwrap();
    let aliases: Vec<&str> = input.aliases.iter().map(|alias| alias.as_str()).collect();
    assert_eq!(aliases, ["www.a.com", "b.com"]);
    assert!(parse(json!({"title": "x", "aliases": ["not_a_host"]})).is_err());
}

#[test]
fn brand_limits_apply() {
    let link = json!({"label": "x", "url": "/x"});
    let links: Vec<Value> = std::iter::repeat_n(link, 9).collect();
    assert!(parse(json!({"title": "x", "brand": {"links": links[..8]}})).is_ok());
    assert!(parse(json!({"title": "x", "brand": {"links": links}})).is_err());
    assert!(parse(json!({"title": "x", "brand": {"head": "a".repeat(20_001)}})).is_err());
    assert!(parse(json!({"title": "x", "brand": {"footer": "é".repeat(500)}})).is_ok());
    assert!(parse(json!({"title": "x", "brand": {"footer": "a".repeat(501)}})).is_err());
    assert!(parse(json!({"title": "x", "brand": {"font": "comic"}})).is_err());
    assert!(parse(json!({"title": "x", "brand": {"accent": "red;}"}})).is_err());
    assert!(parse(json!({"title": "x", "brand": {"logo": "//evil.com/a.png"}})).is_err());
}

#[test]
fn unknown_fields_fail() {
    assert!(parse(json!({"title": "x", "accent": "#abc"})).is_err());
    assert!(parse(json!({"title": "x", "brand": {"color": "#abc"}})).is_err());
}

#[test]
fn urls_are_checked() {
    assert!(parse(json!({"title": "x", "project_url": "javascript:alert(1)"})).is_err());
    assert!(parse(json!({"title": "x", "author_url": "/relative"})).is_err());
    assert!(parse(json!({"title": "x", "project_url": "https://harbor.example.com"})).is_ok());
    assert!(parse(json!({"title": "x", "indexnow_key": "../x.txt"})).is_err());
    assert!(parse(json!({"title": "x", "home": "api"})).is_err());
}

#[test]
fn font_parses_from_text() {
    assert_eq!("sans".parse::<Font>(), Ok(Font::Sans));
    assert!("comic".parse::<Font>().is_err());
    assert_eq!(Font::Serif.to_string(), "serif");
}

#[test]
fn indexnow_defaults_on_and_announcement_off() {
    let input = parse(json!({"title": "Harbor Notes"})).unwrap();
    assert!(input.indexnow);
    assert_eq!(input.announcement, None);
    let input = parse(json!({"title": "x", "indexnow": false})).unwrap();
    assert!(!input.indexnow);
}

#[test]
fn social_links_are_limited_and_unique() {
    let link = json!({"network": "github", "url": "https://github.com/maralind"});
    assert!(parse(json!({"title": "x", "brand": {"social": [link]}})).is_ok());
    assert!(parse(json!({"title": "x", "brand": {"social": [link, link]}})).is_err());
    let many: Vec<Value> = (0..13)
        .map(|index| json!({"network": "github", "url": format!("https://github.com/u{index}")}))
        .collect();
    assert!(parse(json!({"title": "x", "brand": {"social": many}})).is_err());
}
