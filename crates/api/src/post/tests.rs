use serde_json::{Value, json};

use crate::post::{Kind, PostInput, Status};

fn parse(value: Value) -> Result<PostInput, serde_json::Error> {
    serde_json::from_value(value)
}

#[test]
fn omitted_fields_take_defaults() {
    let input = parse(json!({"title": "Hello", "body": "Text."})).unwrap();
    assert_eq!(input.kind, Kind::Post);
    assert_eq!(input.status, Status::Draft);
    assert!(input.tags.as_slice().is_empty());
    assert_eq!(input.published_at, None);
    assert_eq!(input.show_cta, None);
}

#[test]
fn title_and_body_are_required() {
    assert!(parse(json!({"body": "Text."})).is_err());
    assert!(parse(json!({"title": "Hello"})).is_err());
    assert!(parse(json!({"title": " ", "body": "Text."})).is_err());
    assert!(parse(json!({"title": "Hello", "body": " \n"})).is_err());
    assert!(parse(json!({"title": "a".repeat(301), "body": "Text."})).is_err());
    assert!(parse(json!({"title": "a".repeat(300), "body": "Text."})).is_ok());
}

#[test]
fn enums_and_urls_are_checked() {
    assert!(parse(json!({"title": "x", "body": "x", "kind": "note"})).is_err());
    assert!(parse(json!({"title": "x", "body": "x", "status": "live"})).is_err());
    assert!(parse(json!({"title": "x", "body": "x", "cover": "//x"})).is_err());
    assert!(parse(json!({"title": "x", "body": "x", "og_image": "/media/own.png"})).is_ok());
    assert!(parse(json!({"title": "x", "body": "x", "extra": true})).is_err());
}

#[test]
fn tags_are_deduplicated_and_limited() {
    let input = parse(json!({"title": "x", "body": "x", "tags": ["A", " a ", "B"]})).unwrap();
    let names: Vec<&str> = input
        .tags
        .as_slice()
        .iter()
        .map(|tag| tag.as_str())
        .collect();
    assert_eq!(names, ["A", "B"]);
    let many: Vec<String> = (0..21).map(|n| format!("t{n}")).collect();
    assert!(parse(json!({"title": "x", "body": "x", "tags": many})).is_err());
}

#[test]
fn enums_round_trip_through_text() {
    assert_eq!("page".parse::<Kind>(), Ok(Kind::Page));
    assert_eq!("published".parse::<Status>(), Ok(Status::Published));
    assert_eq!(Kind::Page.to_string(), "page");
    assert_eq!(Status::Draft.as_str(), "draft");
}

#[test]
fn control_characters_are_rejected() {
    for field in ["title", "description", "body"] {
        let mut post = json!({"title": "x", "body": "x"});
        post[field] = json!("bad\u{1b}text");
        assert!(parse(post).is_err(), "{field}");
    }
    let tags = json!({"title": "x", "body": "x", "tags": ["ok", "bell\u{7}"]});
    assert!(parse(tags).is_err());
    let text = json!({"title": "x", "body": "line one\nline\ttwo\r\n", "description": "a\tb"});
    assert!(parse(text).is_ok());
}

#[test]
fn publication_time_is_in_range() {
    for time in [0_i64, 1_793_093_400, 253_402_300_799] {
        let post = parse(json!({"title": "x", "body": "x", "published_at": time})).unwrap();
        assert_eq!(post.published_at, Some(time));
    }
    for time in [-1_i64, 253_402_300_800, 1_793_093_400_000] {
        assert!(
            parse(json!({"title": "x", "body": "x", "published_at": time})).is_err(),
            "{time}"
        );
    }
    assert!(parse(json!({"title": "x", "body": "x", "published_at": null})).is_ok());
}

#[test]
fn show_cta_round_trips() {
    for value in [Value::Null, json!(true), json!(false)] {
        let post = parse(json!({"title": "x", "body": "x", "show_cta": value})).unwrap();
        assert_eq!(serde_json::to_value(&post).unwrap()["show_cta"], value);
    }
    assert!(parse(json!({"title": "x", "body": "x", "show_cta": "yes"})).is_err());
}
