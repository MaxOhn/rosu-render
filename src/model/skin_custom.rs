use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SkinInfo {
    /// The name of the skin.
    #[serde(rename = "skinName")]
    pub name: Box<str>,
    /// The author (skinner, parsed from the skin's skin.ini) of the skin.
    #[serde(rename = "skinAuthor")]
    pub author: Box<str>,
    /// The download link for this custom skin, from issou.best servers.
    #[serde(rename = "downloadLink")]
    pub download_link: Box<str>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SkinDeleted {
    /// true if found, false if not.
    pub found: bool,
    /// true if removed, false if not.
    pub removed: bool,
    /// The info message, in english, of an error.
    pub message: Box<str>,
    /// The name of the skin.
    pub name: Box<str>,
    /// The author (skinner, parsed from the skin's skin.ini) of the skin.
    pub author: Box<str>,
}
