//! The look settings of a site.

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

use crate::{
    site::{
        accent::Accent,
        link::{Link, LinkUrl},
        social::SocialLink,
        theme::{CardStyle, CodeTheme, Font, HomeLayout, Palette, Theme, Width},
    },
    text,
};

/// The look settings of a site, shown as "Appearance" in the console.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(default, deny_unknown_fields)]
pub struct Brand {
    pub accent: Accent,
    /// The fallback font stack, and the whole page font when `font_family` is `None`.
    pub font: Font,
    /// A family of the font library for the page text, 1 to 100 characters.
    /// `None` keeps the system font stack of `font`.
    #[serde(deserialize_with = "de_font_family")]
    pub font_family: Option<String>,
    pub logo: Option<LinkUrl>,
    pub favicon: Option<LinkUrl>,
    /// The share image for pages that have no share image of their own.
    pub og_image: Option<LinkUrl>,
    /// Header links, at most 8.
    #[serde(deserialize_with = "text::de_max_items::<_, Link, LINKS_MAX>")]
    pub links: Vec<Link>,
    /// Raw HTML for the page head, at most 20 000 bytes.
    #[serde(deserialize_with = "text::de_max_bytes::<_, HEAD_MAX>")]
    pub head: String,
    /// Footer text, at most 500 characters.
    #[serde(deserialize_with = "text::de_max_chars::<_, FOOTER_MAX>")]
    pub footer: String,
    /// Shows the "Published with Cuhlumn" credit, with a link to the GitHub repository, in the footer.
    pub credit: bool,
    pub palette: Palette,
    pub theme: Theme,
    /// Shows the button that switches between the light and dark themes.
    pub theme_toggle: bool,
    pub width: Width,
    pub home_layout: HomeLayout,
    /// Profile links, at most 12, without repetitions.
    #[serde(deserialize_with = "de_social")]
    pub social: Vec<SocialLink>,
    /// Shows the profile links in the header as well as in the footer.
    pub social_in_header: bool,
    pub code_theme: CodeTheme,
    pub card_style: CardStyle,
    /// Shows the date and the reading time on share images.
    pub card_meta: bool,
}

const LINKS_MAX: usize = 8;
const HEAD_MAX: usize = 20_000;
const FOOTER_MAX: usize = 500;
const SOCIAL_MAX: usize = 12;
const FONT_FAMILY_MAX: usize = 100;

impl Default for Brand {
    fn default() -> Self {
        Self {
            accent: Accent::default(),
            font: Font::default(),
            font_family: None,
            logo: None,
            favicon: None,
            og_image: None,
            links: Vec::new(),
            head: String::new(),
            footer: String::new(),
            credit: true,
            palette: Palette::default(),
            theme: Theme::default(),
            theme_toggle: true,
            width: Width::default(),
            home_layout: HomeLayout::default(),
            social: Vec::new(),
            social_in_header: false,
            code_theme: CodeTheme::default(),
            card_style: CardStyle::default(),
            card_meta: true,
        }
    }
}

fn de_social<'de, D>(deserializer: D) -> Result<Vec<SocialLink>, D::Error>
where
    D: Deserializer<'de>,
{
    let links = text::de_max_items::<_, SocialLink, SOCIAL_MAX>(deserializer)?;
    for (index, link) in links.iter().enumerate() {
        if links[..index].contains(link) {
            return Err(D::Error::custom(format!(
                "the {} link {} is repeated",
                link.network, link.url
            )));
        }
    }
    Ok(links)
}

/// Deserializes a library family name: trimmed, 1 to 100 characters, without any control character.
fn de_font_family<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(value) = Option::<String>::deserialize(deserializer)? else {
        return Ok(None);
    };
    let value = text::required(&value, FONT_FAMILY_MAX).map_err(D::Error::custom)?;
    match value.chars().find(|character| character.is_control()) {
        Some(character) => Err(D::Error::custom(text::TextError::ControlChar {
            code: u32::from(character),
        })),
        None => Ok(Some(value)),
    }
}
