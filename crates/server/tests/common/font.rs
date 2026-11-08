//! A minimal TrueType font builder and the font upload request.

use axum::body::Body;

use crate::common::{ADMIN_HOST, Res, TOKEN, TestApp, request};

/// The weight and slant of a test face.
#[derive(Clone, Copy, Debug)]
pub struct FaceStyle {
    pub weight: u16,
    pub italic: bool,
}

pub const REGULAR: FaceStyle = FaceStyle {
    weight: 400,
    italic: false,
};

/// Uploads font bytes to the font library.
pub async fn upload(app: &TestApp, body: Vec<u8>) -> Res {
    let builder = request("POST", ADMIN_HOST, "/api/fonts")
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "font/ttf");
    app.send(builder.body(Body::from(body)).unwrap()).await
}

/// Builds a minimal TrueType file with the tables `OS/2`, `head`, `hhea`, `maxp`, and `name`.
pub fn minimal_ttf(family: &str, style: FaceStyle) -> Vec<u8> {
    let tables: [(&[u8; 4], Vec<u8>); 5] = [
        (b"OS/2", os2(style)),
        (b"head", head()),
        (b"hhea", hhea()),
        (b"maxp", [0, 0, 0x50, 0, 0, 1].to_vec()),
        (b"name", name(family, style)),
    ];
    let mut font = Vec::new();
    font.extend_from_slice(&[0, 1, 0, 0]);
    font.extend_from_slice(&5_u16.to_be_bytes());
    font.extend_from_slice(&[0, 64, 0, 2, 0, 16]);
    let mut offset = 12 + 16 * tables.len();
    let mut data = Vec::new();
    for (tag, table) in &tables {
        font.extend_from_slice(*tag);
        font.extend_from_slice(&0_u32.to_be_bytes());
        font.extend_from_slice(&u32::try_from(offset).unwrap().to_be_bytes());
        font.extend_from_slice(&u32::try_from(table.len()).unwrap().to_be_bytes());
        let mut padded = table.clone();
        padded.resize(table.len().div_ceil(4) * 4, 0);
        offset += padded.len();
        data.extend(padded);
    }
    font.extend(data);
    font
}

/// Returns a version 0 `OS/2` table with the weight class and the italic or regular selection flag.
fn os2(style: FaceStyle) -> Vec<u8> {
    let mut table = vec![0; 78];
    table[4..6].copy_from_slice(&style.weight.to_be_bytes());
    table[6..8].copy_from_slice(&5_u16.to_be_bytes());
    let selection: u16 = if style.italic { 1 } else { 1 << 6 };
    table[62..64].copy_from_slice(&selection.to_be_bytes());
    table
}

fn head() -> Vec<u8> {
    let mut table = Vec::new();
    table.extend_from_slice(&[0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
    table.extend_from_slice(&0x5F0F_3CF5_u32.to_be_bytes());
    table.extend_from_slice(&0_u16.to_be_bytes());
    table.extend_from_slice(&1000_u16.to_be_bytes());
    table.extend_from_slice(&[0; 16]);
    table.extend_from_slice(&[0, 0, 0, 0, 0x03, 0xE8, 0x03, 0xE8]);
    table.extend_from_slice(&[0, 0, 0, 8, 0, 2, 0, 0, 0, 0]);
    assert_eq!(table.len(), 54);
    table
}

fn hhea() -> Vec<u8> {
    let mut table = vec![0, 1, 0, 0];
    table.extend_from_slice(&800_i16.to_be_bytes());
    table.extend_from_slice(&(-200_i16).to_be_bytes());
    table.extend_from_slice(&[0; 26]);
    table.extend_from_slice(&1_u16.to_be_bytes());
    assert_eq!(table.len(), 36);
    table
}

fn name(family: &str, style: FaceStyle) -> Vec<u8> {
    let subfamily = match (style.weight >= 700, style.italic) {
        (false, false) => "Regular",
        (false, true) => "Italic",
        (true, false) => "Bold",
        (true, true) => "BoldItalic",
    };
    let postscript = format!("{}-{subfamily}", family.replace(' ', ""));
    let strings: Vec<(u16, Vec<u8>)> = [
        (1, family),
        (2, subfamily),
        (4, family),
        (6, postscript.as_str()),
    ]
    .into_iter()
    .map(|(id, text)| (id, text.encode_utf16().flat_map(u16::to_be_bytes).collect()))
    .collect();
    let count = u16::try_from(strings.len()).unwrap();
    let mut table = Vec::new();
    table.extend_from_slice(&0_u16.to_be_bytes());
    table.extend_from_slice(&count.to_be_bytes());
    table.extend_from_slice(&(6 + 12 * count).to_be_bytes());
    let mut storage = Vec::new();
    for (id, text) in &strings {
        for value in [3_u16, 1, 0x0409, *id] {
            table.extend_from_slice(&value.to_be_bytes());
        }
        table.extend_from_slice(&u16::try_from(text.len()).unwrap().to_be_bytes());
        table.extend_from_slice(&u16::try_from(storage.len()).unwrap().to_be_bytes());
        storage.extend_from_slice(text);
    }
    table.extend(storage);
    table
}
