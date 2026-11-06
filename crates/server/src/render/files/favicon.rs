//! The generated favicon: a round accent disc with the site initial in a serif letter.

use cuhlumn_api::site::Site;

use crate::render::{escape, initial, style::Rgb};

/// Returns the favicon SVG of a site. The letter is white or black, whichever reads better on the accent.
#[must_use]
pub fn render(site: &Site) -> String {
    let accent = &site.brand.accent;
    let ink = Rgb::from_accent(accent).readable_text().hex();
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 32 32\">\
         <circle cx=\"16\" cy=\"16\" r=\"16\" fill=\"{accent}\"/>\
         <text x=\"16\" y=\"16\" dy=\".35em\" text-anchor=\"middle\" fill=\"{ink}\" font-size=\"19\" font-weight=\"600\" \
         font-family=\"Georgia,'Iowan Old Style',Palatino,serif\">{}</text></svg>",
        escape(&initial(&site.title))
    )
}
