use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::util::datetime::deserialize_datetime;

/// A list of [`Render`].
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RenderList {
    /// Array of renders returned by the api
    pub renders: Vec<Render>,
    /// The total number of renders on o!rdr,
    /// but if search query the total numbers of renders corresponding to that query will be used.
    #[serde(rename = "maxRenders")]
    pub max_renders: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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
    pub skin: Box<str>,
    #[serde(rename = "hasCursorMiddle")]
    pub has_cursor_middle: bool,
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
}

/// The response of the server when the render got created successfully.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct RenderCreated {
    /// The render ID of your render that got created.
    #[serde(rename = "renderID")]
    pub render_id: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum RenderResolution {
    /// 720x480 (30fps)
    SD480,
    /// 960x540 (30fps)
    SD960,
    /// 1280x720 (60fps)
    HD720,
    /// 1920x1080 (60fps)
    HD1080,
}

impl RenderResolution {
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

#[derive(Clone, Debug, Serialize)]
pub struct RenderOptions {
    pub resolution: RenderResolution,
    #[serde(rename = "globalVolume")]
    pub global_volume: u8,
    #[serde(rename = "musicVolume")]
    pub music_volume: u8,
    #[serde(rename = "hitsoundVolume")]
    pub hitsound_volume: u8,
    #[serde(rename = "showHitErrorMeter")]
    pub show_hit_error_meter: bool,
    #[serde(rename = "showUnstableRate")]
    pub show_unstable_rate: bool,
    #[serde(rename = "showScore")]
    pub show_score: bool,
    #[serde(rename = "showHPBar")]
    pub show_hp_bar: bool,
    #[serde(rename = "showComboCounter")]
    pub show_combo_counter: bool,
    #[serde(rename = "showPPCounter")]
    pub show_pp_counter: bool,
    #[serde(rename = "showKeyOverlay")]
    pub show_key_overlay: bool,
    #[serde(rename = "showScoreboard")]
    pub show_scoreboard: bool,
    #[serde(rename = "showBorders")]
    pub show_borders: bool,
    #[serde(rename = "showMods")]
    pub show_mods: bool,
    #[serde(rename = "showResultScreen")]
    pub show_result_screen: bool,
    #[serde(rename = "useSkinCursor")]
    pub use_skin_cursor: bool,
    #[serde(rename = "useSkinHitsounds")]
    pub use_skin_hitsounds: bool,
    #[serde(rename = "useBeatmapColors")]
    pub use_beatmap_colors: bool,
    #[serde(rename = "cursorScaleToCS")]
    pub cursor_scale_to_cs: bool,
    #[serde(rename = "cursorRainbow")]
    pub cursor_rainbow: bool,
    #[serde(rename = "cursorTrailGlow")]
    pub cursor_trail_glow: bool,
    #[serde(rename = "drawFollowPoints")]
    pub draw_follow_points: bool,
    #[serde(rename = "drawComboNumbers")]
    pub draw_combo_numbers: bool,
    #[serde(rename = "cursorSize")]
    pub cursor_size: f32,
    #[serde(rename = "cursorTrail")]
    pub cursor_trail: bool,
    #[serde(rename = "scaleToTheBeat")]
    pub beat_scaling: bool,
    #[serde(rename = "sliderMerge")]
    pub slider_merge: bool,
    #[serde(rename = "objectsRainbow")]
    pub objects_rainbow: bool,
    #[serde(rename = "objectsFlashToTheBeat")]
    pub flash_objects: bool,
    #[serde(rename = "useHitCircleColor")]
    pub use_slider_hitcircle_color: bool,
    #[serde(rename = "seizureWarning")]
    pub seizure_warning: bool,
    #[serde(rename = "loadStoryboard")]
    pub load_storyboard: bool,
    #[serde(rename = "loadVideo")]
    pub load_video: bool,
    #[serde(rename = "introBGDim")]
    pub intro_bg_dim: u8,
    #[serde(rename = "inGameBGDim")]
    pub ingame_bg_dim: u8,
    #[serde(rename = "breakBGDim")]
    pub break_bg_dim: u8,
    #[serde(rename = "BGParallax")]
    pub bg_parallax: bool,
    #[serde(rename = "showDanserLogo")]
    pub show_danser_logo: bool,
    #[serde(rename = "skip")]
    pub skip_intro: bool,
    #[serde(rename = "cursorRipples")]
    pub cursor_ripples: bool,
    #[serde(rename = "sliderSnakingIn")]
    pub slider_snaking_in: bool,
    #[serde(rename = "sliderSnakingOut")]
    pub slider_snaking_out: bool,
    #[serde(rename = "showHitCounter")]
    pub show_hit_counter: bool,
    #[serde(rename = "showAvatarsOnScoreboard")]
    pub show_avatars_on_scoreboard: bool,
    #[serde(rename = "showAimErrorMeter")]
    pub show_aim_error_meter: bool,
    #[serde(rename = "playNightcoreSamples")]
    pub play_nightcore_samples: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            resolution: RenderResolution::HD720,
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
            use_slider_hitcircle_color: true,
            seizure_warning: false,
            load_storyboard: true,
            load_video: true,
            intro_bg_dim: 0,
            ingame_bg_dim: 75,
            break_bg_dim: 30,
            bg_parallax: false,
            show_danser_logo: true,
            skip_intro: true,
            cursor_ripples: false,
            slider_snaking_in: true,
            slider_snaking_out: true,
            show_hit_counter: false,
            show_avatars_on_scoreboard: false,
            show_aim_error_meter: false,
            play_nightcore_samples: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderSkinOption<'a> {
    pub skin_name: &'a str,
    pub is_custom: bool,
}

impl<'a> RenderSkinOption<'a> {
    pub fn new(skin_name: &'a str, is_custom: bool) -> Self {
        Self {
            skin_name,
            is_custom,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RenderServers {
    pub servers: Vec<RenderServer>,
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
