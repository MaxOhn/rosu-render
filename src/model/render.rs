use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result as FmtResult},
};

use hyper::{body::Bytes, StatusCode};
use serde::{
    de::{Error as DeError, IgnoredAny, MapAccess, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
};
use time::OffsetDateTime;

use crate::{request::Requestable, util::datetime::deserialize_datetime, ClientError};

/// A list of [`Render`].
#[derive(Clone, Debug, Deserialize)]
pub struct RenderList {
    /// Array of renders returned by the api
    pub renders: Vec<Render>,
    /// The total number of renders on o!rdr,
    /// but if search query the total numbers of renders corresponding to that query will be used.
    #[serde(rename = "maxRenders")]
    pub max_renders: u32,
}

impl Requestable for RenderList {
    fn response_error(status: StatusCode, bytes: Bytes) -> ClientError {
        ClientError::response_error(bytes, status.as_u16())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Render {
    #[serde(rename = "renderID")]
    pub id: u32,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub date: OffsetDateTime,
    pub username: Box<str>,
    pub progress: Box<str>,
    pub renderer: Box<str>,
    pub description: Box<str>,
    pub title: Box<str>,
    #[serde(rename = "isBot")]
    pub is_bot: bool,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "videoUrl")]
    pub video_url: Box<str>,
    #[serde(rename = "mapLink")]
    pub map_link: Box<str>,
    #[serde(rename = "mapTitle")]
    pub map_title: Box<str>,
    #[serde(rename = "replayDifficulty")]
    pub replay_difficulty: Box<str>,
    #[serde(rename = "replayUsername")]
    pub replay_username: Box<str>,
    #[serde(rename = "mapID")]
    pub map_id: u32,
    #[serde(rename = "needToRedownload")]
    pub need_to_redownload: bool,
    #[serde(rename = "motionBlur960fps")]
    pub motion_blur: bool,
    #[serde(rename = "renderStartTime", deserialize_with = "deserialize_datetime")]
    pub render_start_time: OffsetDateTime,
    #[serde(rename = "renderEndTime", deserialize_with = "deserialize_datetime")]
    pub render_end_time: OffsetDateTime,
    #[serde(rename = "uploadEndTime", deserialize_with = "deserialize_datetime")]
    pub upload_end_time: OffsetDateTime,
    #[serde(rename = "renderTotalTime")]
    pub render_total_time: u32,
    #[serde(rename = "uploadTotalTime")]
    pub upload_total_time: u32,
    #[serde(rename = "mapLength")]
    pub map_length: u32,
    #[serde(rename = "replayMods")]
    pub replay_mods: Box<str>,
    pub removed: bool,
    #[serde(flatten)]
    pub options: RenderOptions,
    #[serde(flatten)]
    pub skin: RenderSkinOption<'static>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum RenderResolution {
    /// 720x480 (30fps)
    #[serde(rename = "720x480")]
    SD480,
    /// 960x540 (30fps)
    #[serde(rename = "960x540")]
    SD960,
    /// 1280x720 (60fps)
    #[serde(rename = "1280x720")]
    HD720,
    /// 1920x1080 (60fps)
    #[serde(rename = "1920x1080")]
    HD1080,
}

impl RenderResolution {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SD480 => "720x480",
            Self::SD960 => "960x540",
            Self::HD720 => "1280x720",
            Self::HD1080 => "1920x1080",
        }
    }
}

impl Display for RenderResolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.as_str())
    }
}

/// Customize danser settings when rendering.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RenderOptions {
    pub resolution: RenderResolution,
    /// The global volume for the video, in percent, from 0 to 100.
    #[serde(rename = "globalVolume")]
    pub global_volume: u8,
    /// The music volume for the video, in percent, from 0 to 100.
    #[serde(rename = "musicVolume")]
    pub music_volume: u8,
    /// The hitsounds volume for the video, in percent, from 0 to 100.
    #[serde(rename = "hitsoundVolume")]
    pub hitsound_volume: u8,
    /// Show the hit error meter.
    #[serde(rename = "showHitErrorMeter")]
    pub show_hit_error_meter: bool,
    /// Show the unstable rate, only takes effect if `show_hit_error_meter` is set to true.
    #[serde(rename = "showUnstableRate")]
    pub show_unstable_rate: bool,
    /// Show the score.
    #[serde(rename = "showScore")]
    pub show_score: bool,
    /// Show the HP bar.
    #[serde(rename = "showHPBar")]
    pub show_hp_bar: bool,
    /// Show the combo counter.
    #[serde(rename = "showComboCounter")]
    pub show_combo_counter: bool,
    /// Show the PP Counter or not.
    #[serde(rename = "showPPCounter")]
    pub show_pp_counter: bool,
    /// Show the scoreboard or not.
    #[serde(rename = "showScoreboard")]
    pub show_scoreboard: bool,
    /// Show the playfield borders or not.
    #[serde(rename = "showBorders")]
    pub show_borders: bool,
    /// Show the mods used during the game or not.
    #[serde(rename = "showMods")]
    pub show_mods: bool,
    /// Show the result screen or not.
    #[serde(rename = "showResultScreen")]
    pub show_result_screen: bool,
    /// Use the skin cursor or not. If not, danser cursor will be used.
    #[serde(rename = "useSkinCursor")]
    pub use_skin_cursor: bool,
    /// Use the skin combo colors or not.
    #[serde(rename = "useSkinColors")]
    pub use_skin_colors: bool,
    /// Use skin hitsounds, if false beatmap hitsounds will be used.
    #[serde(rename = "useSkinHitsounds")]
    pub use_skin_hitsounds: bool,
    /// Use the beatmap combo colors or not, overrides useSkinColors if true.
    #[serde(rename = "useBeatmapColors")]
    pub use_beatmap_colors: bool,
    /// Scale cursor to circle size. Does not do anything at the moment.
    #[serde(rename = "cursorScaleToCS")]
    pub cursor_scale_to_cs: bool,
    /// Makes the cursor rainbow, only takes effect if `use_skin_cursor` is set to false.
    #[serde(rename = "cursorRainbow")]
    pub cursor_rainbow: bool,
    /// Have a glow with the trail or not.
    #[serde(rename = "cursorTrailGlow")]
    pub cursor_trail_glow: bool,
    /// Draw follow points between objects or not.
    #[serde(rename = "drawFollowPoints")]
    pub draw_follow_points: bool,
    /// Scale objects to the beat.
    #[serde(rename = "scaleToTheBeat")]
    pub beat_scaling: bool,
    /// Merge sliders or not.
    #[serde(rename = "sliderMerge")]
    pub slider_merge: bool,
    /// Makes the objects rainbow, overrides `use_skin_colors` and `use_beatmap_colors`.
    #[serde(rename = "objectsRainbow")]
    pub objects_rainbow: bool,
    /// Makes the objects flash to the beat.
    #[serde(rename = "objectsFlashToTheBeat")]
    pub flash_objects: bool,
    /// Makes the slider body have the same color as the hit circles.
    #[serde(rename = "useHitCircleColor")]
    pub use_slider_hitcircle_color: bool,
    /// Display a 5 second seizure warning before the video.
    #[serde(rename = "seizureWarning")]
    pub seizure_warning: bool,
    /// Load the background storyboard.
    #[serde(rename = "loadStoryboard")]
    pub load_storyboard: bool,
    /// Load the background video (`load_storyboard` has to be set to true).
    #[serde(rename = "loadVideo")]
    pub load_video: bool,
    /// Background dim for the intro, in percent, from 0 to 100.
    #[serde(rename = "introBGDim")]
    pub intro_bg_dim: u8,
    /// Background dim in game, in percent, from 0 to 100.
    #[serde(rename = "inGameBGDim")]
    pub ingame_bg_dim: u8,
    /// Background dim in break, in percent, from 0 to 100.
    #[serde(rename = "breakBGDim")]
    pub break_bg_dim: u8,
    /// Adds a parallax effect.
    #[serde(rename = "BGParallax")]
    pub bg_parallax: bool,
    /// Show danser logo on the intro.
    #[serde(rename = "showDanserLogo")]
    pub show_danser_logo: bool,
    /// Skip the intro or not.
    #[serde(rename = "skip")]
    pub skip_intro: bool,
    /// Show cursor ripples when keypress.
    #[serde(rename = "cursorRipples")]
    pub cursor_ripples: bool,
    /// Set the cursor size, multiplier from 0.5 to 2.
    #[serde(rename = "cursorSize")]
    pub cursor_size: f32,
    /// Show the cursor trail or not.
    #[serde(rename = "cursorTrail")]
    pub cursor_trail: bool,
    /// Show the combo numbers in objects.
    #[serde(rename = "drawComboNumbers")]
    pub draw_combo_numbers: bool,
    /// Have slider snaking in.
    #[serde(rename = "sliderSnakingIn")]
    pub slider_snaking_in: bool,
    /// Have slider snaking out.
    #[serde(rename = "sliderSnakingOut")]
    pub slider_snaking_out: bool,
    /// Shows a hit counter (100, 50, miss) below the PP counter.
    #[serde(rename = "showHitCounter")]
    pub show_hit_counter: bool,
    /// Show the key overlay or not.
    #[serde(rename = "showKeyOverlay")]
    pub show_key_overlay: bool,
    /// Show avatars on the left of the username of a player on the scoreboard.
    /// May break some skins because the width of the scoreboard increases.
    #[serde(rename = "showAvatarsOnScoreboard")]
    pub show_avatars_on_scoreboard: bool,
    /// Show the Aim Error Meter or not.
    #[serde(rename = "showAimErrorMeter")]
    pub show_aim_error_meter: bool,
    /// Play nightcore hitsounds or not.
    #[serde(rename = "playNightcoreSamples")]
    pub play_nightcore_samples: bool,
    /// Show the strain graph or not.
    #[serde(rename = "showStrainGraph")]
    pub show_strain_graph: bool,
    /// Show the slider breaks count in the hit counter.
    #[serde(rename = "showSliderBreaks")]
    pub show_slider_breaks: bool,
    /// Ignores fail in the replay or not.
    #[serde(rename = "ignoreFail")]
    pub ignore_fail: bool,
}

impl RenderOptions {
    pub const DEFAULT_RESOLUTION: RenderResolution = RenderResolution::HD720;
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            resolution: Self::DEFAULT_RESOLUTION,
            global_volume: 50,
            music_volume: 50,
            hitsound_volume: 50,
            show_hit_error_meter: true,
            show_unstable_rate: true,
            show_score: true,
            show_hp_bar: true,
            show_combo_counter: true,
            show_pp_counter: true,
            show_key_overlay: true,
            show_scoreboard: true,
            show_borders: true,
            show_mods: true,
            show_result_screen: true,
            use_skin_cursor: true,
            use_skin_colors: false,
            use_skin_hitsounds: true,
            use_beatmap_colors: true,
            cursor_scale_to_cs: false,
            cursor_rainbow: false,
            cursor_trail_glow: false,
            draw_follow_points: true,
            draw_combo_numbers: true,
            cursor_size: 1.0,
            cursor_trail: true,
            beat_scaling: false,
            slider_merge: false,
            objects_rainbow: false,
            flash_objects: false,
            use_slider_hitcircle_color: false,
            seizure_warning: false,
            load_storyboard: false,
            load_video: false,
            intro_bg_dim: 0,
            ingame_bg_dim: 80,
            break_bg_dim: 30,
            bg_parallax: false,
            show_danser_logo: true,
            skip_intro: true,
            cursor_ripples: false,
            slider_snaking_in: true,
            slider_snaking_out: true,
            show_hit_counter: true,
            show_avatars_on_scoreboard: false,
            show_aim_error_meter: false,
            play_nightcore_samples: true,
            show_strain_graph: false,
            show_slider_breaks: false,
            ignore_fail: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderSkinOption<'a> {
    Official { name: Cow<'a, str> },
    Custom { id: u32 },
}

impl<'a> Default for RenderSkinOption<'a> {
    fn default() -> Self {
        Self::Official {
            name: "default".into(),
        }
    }
}

impl<'a> From<u32> for RenderSkinOption<'a> {
    fn from(id: u32) -> Self {
        Self::Custom { id }
    }
}

macro_rules! impl_from_name {
    ( $( $ty:ty ),* ) => {
        $(
            impl<'a> From<$ty> for RenderSkinOption<'a> {
                fn from(name: $ty) -> Self {
                    Self::Official { name: name.into() }
                }
            }
        )*
    };
}

impl_from_name!(&'a str, &'a String, String, Cow<'a, str>);

impl<'de> Deserialize<'de> for RenderSkinOption<'static> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct SkinVisitor;

        impl<'de> Visitor<'de> for SkinVisitor {
            type Value = RenderSkinOption<'static>;

            fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.write_str("`skin` and `customSkin` fields")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut custom_skin: Option<bool> = None;
                let mut skin: Option<String> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "customSkin" => custom_skin = Some(map.next_value()?),
                        "skin" => skin = Some(map.next_value()?),
                        _ => {
                            let _: IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let custom_skin =
                    custom_skin.ok_or_else(|| DeError::missing_field("customSkin"))?;
                let skin = skin.ok_or_else(|| DeError::missing_field("skin"))?;

                let skin = if custom_skin {
                    let id = skin
                        .parse()
                        .map_err(|_| DeError::invalid_value(Unexpected::Str(&skin), &"a u32"))?;

                    RenderSkinOption::Custom { id }
                } else {
                    RenderSkinOption::Official {
                        name: Cow::Owned(skin),
                    }
                };

                Ok(skin)
            }
        }

        d.deserialize_map(SkinVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RenderServers {
    pub servers: Vec<RenderServer>,
}

impl Requestable for RenderServers {
    fn response_error(status: StatusCode, bytes: Bytes) -> ClientError {
        ClientError::response_error(bytes, status.as_u16())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RenderServer {
    pub enabled: bool,
    #[serde(rename = "lastSeen", deserialize_with = "deserialize_datetime")]
    pub last_seen: OffsetDateTime,
    pub name: Box<str>,
    pub priority: f32,
    #[serde(rename = "oldScore")]
    pub old_score: f32,
    #[serde(rename = "avgFPS")]
    pub avg_fps: u32,
    pub power: Box<str>,
    pub status: Box<str>,
    #[serde(rename = "totalRendered")]
    pub total_rendered: u32,
    #[serde(rename = "renderingType")]
    pub rendering_type: Box<str>,
    pub cpu: Box<str>,
    pub gpu: Box<str>,
    #[serde(rename = "motionBlurCapable")]
    pub motion_blur_capable: bool,
    #[serde(rename = "usingOsuApi")]
    pub using_osu_api: bool,
    #[serde(rename = "uhdCapable")]
    pub uhd_capable: bool,
    #[serde(rename = "avgRenderTime")]
    pub avg_render_time: f32,
    #[serde(rename = "avgUploadTime")]
    pub avg_upload_time: f32,
    #[serde(rename = "totalAvgTime")]
    pub total_avg_time: f32,
    #[serde(rename = "totalUploadedVideosSize")]
    pub total_uploaded_videos_size: u32,
    #[serde(rename = "ownerUserId")]
    pub owner_user_id: u32,
    #[serde(rename = "ownerUsername")]
    pub owner_username: Box<str>,
    pub customization: RenderServerOptions,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RenderServerOptions {
    #[serde(rename = "textColor")]
    pub text_color: Box<str>,
    #[serde(rename = "backgroundType")]
    pub background_type: i32,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServerOnlineCount(pub u32);

impl Requestable for ServerOnlineCount {
    fn response_error(status: StatusCode, bytes: Bytes) -> ClientError {
        ClientError::response_error(bytes, status.as_u16())
    }
}
