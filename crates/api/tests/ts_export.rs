//! Writes the TypeScript bindings of the wire types to `console/src/types/api/`.

#![cfg(feature = "ts")]

use std::path::PathBuf;

use cuhlumn_api::{
    fonts::{FontItem, GoogleFontInput},
    post::{Post, PostInput},
    settings::{ServerSettings, ServerSettingsInput, TokenInput},
    site::announcement::AnnouncementStatus,
    site::{Site, SiteInput},
    wire::{
        ErrorBody, Health, MediaItem, MediaSaved, PostSaved, Preview, PreviewInput, Session,
        SessionInput,
    },
};
use ts_rs::{Config, TS};

#[test]
fn export_typescript_bindings() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../console/src/types/api");
    let config = Config::new().with_out_dir(&dir);
    Site::export_all(&config).unwrap();
    SiteInput::export_all(&config).unwrap();
    Post::export_all(&config).unwrap();
    PostInput::export_all(&config).unwrap();
    PostSaved::export_all(&config).unwrap();
    MediaSaved::export_all(&config).unwrap();
    MediaItem::export_all(&config).unwrap();
    PreviewInput::export_all(&config).unwrap();
    Preview::export_all(&config).unwrap();
    Session::export_all(&config).unwrap();
    SessionInput::export_all(&config).unwrap();
    Health::export_all(&config).unwrap();
    ErrorBody::export_all(&config).unwrap();
    ServerSettings::export_all(&config).unwrap();
    ServerSettingsInput::export_all(&config).unwrap();
    TokenInput::export_all(&config).unwrap();
    FontItem::export_all(&config).unwrap();
    GoogleFontInput::export_all(&config).unwrap();
    AnnouncementStatus::export_all(&config).unwrap();
    assert!(dir.join("Site.ts").is_file());
    assert!(dir.join("ErrorCode.ts").is_file());
}
