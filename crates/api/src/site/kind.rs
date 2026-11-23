//! The kinds of sites.

text_enum! {
    /// What the server does for a site.
    pub enum SiteKind / SiteKindError ("site kind") {
        /// A blog that the server serves on the domain of the site.
        #[default]
        Blog = "blog",
        /// A website that another server serves. The server only counts its page views and clicks.
        External = "external",
    }
}
