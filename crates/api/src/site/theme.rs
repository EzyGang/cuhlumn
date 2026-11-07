//! Fixed choices of the site appearance.

text_enum! {
    /// The font family of post text.
    pub enum Font / FontError ("font") {
        #[default]
        Serif = "serif",
        Sans = "sans",
    }
}

text_enum! {
    /// The colour set of the site.
    pub enum Palette / PaletteError ("palette") {
        #[default]
        Paper = "paper",
        Slate = "slate",
        Sepia = "sepia",
        Mono = "mono",
        Contrast = "contrast",
    }
}

text_enum! {
    /// The colour theme that the site shows first.
    pub enum Theme / ThemeError ("theme") {
        /// Follows the colour preference of the reader.
        #[default]
        System = "system",
        Light = "light",
        Dark = "dark",
    }
}

text_enum! {
    /// The width of the text column.
    pub enum Width / WidthError ("width") {
        /// 38rem.
        Narrow = "narrow",
        #[default]
        Normal = "normal",
        /// 52rem.
        Wide = "wide",
    }
}

text_enum! {
    /// The layout of the post list on the home page.
    pub enum HomeLayout / HomeLayoutError ("home layout") {
        #[default]
        List = "list",
        /// One line per post with the date and the title.
        Compact = "compact",
        /// A grid that shows the cover image of each post.
        Cards = "cards",
    }
}

text_enum! {
    /// The colours of highlighted code.
    pub enum CodeTheme / CodeThemeError ("code theme") {
        #[default]
        Github = "github",
        Solarized = "solarized",
        Ocean = "ocean",
        /// Turns highlighting off.
        None = "none",
    }
}

text_enum! {
    /// The colours of share images.
    pub enum CardStyle / CardStyleError ("card style") {
        #[default]
        Dark = "dark",
        Light = "light",
        Accent = "accent",
    }
}

#[cfg(test)]
mod tests {
    use super::{CodeTheme, Font, HomeLayout, Palette, Width};

    #[test]
    fn names_round_trip() {
        for palette in Palette::ALL {
            assert_eq!(palette.as_str().parse::<Palette>(), Ok(*palette));
            let json = serde_json::to_value(palette).unwrap();
            assert_eq!(json.as_str(), Some(palette.as_str()));
        }
        assert_eq!(
            serde_json::to_value(HomeLayout::Compact).unwrap(),
            "compact"
        );
        assert_eq!(serde_json::to_value(CodeTheme::None).unwrap(), "none");
        assert_eq!(Width::default(), Width::Normal);
    }

    #[test]
    fn parse_errors_list_valid_names() {
        let error = "comic".parse::<Font>().unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"the font "comic" is not valid: use serif or sans"#
        );
        let error = "x".parse::<Width>().unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"the width "x" is not valid: use narrow, normal, or wide"#
        );
        assert!(serde_json::from_str::<Palette>(r#""Paper""#).is_err());
    }
}
