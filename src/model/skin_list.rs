use serde::Deserialize;

/// A list of [`Skin`].
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SkinList {
    /// Array of skins returned by the api.
    pub skins: Vec<Skin>,
    /// The total number of skins that are available on o!rdr,
    /// but if search query the total numbers of renders corresponding to that query will be used.
    #[serde(rename = "maxSkins")]
    pub max_skins: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub skin: Box<str>,
    pub presentation_name: Box<str>,
    pub url: Box<str>,
    pub high_res_preview: Box<str>,
    pub low_res_preview: Box<str>,
    pub grid_preview: Box<str>,
    pub id: u32,
    pub has_cursor_middle: bool,
    pub author: Box<str>,
    pub modified: bool,
    pub version: Box<str>,
    pub alphabetical_id: u32,
    pub times_used: u32,
}
