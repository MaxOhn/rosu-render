use std::future::IntoFuture;

use crate::{
    model::{RenderAdded, RenderOptions, RenderSkinOption},
    routing::Route,
    util::multipart::Form,
    ClientError, OrdrClient,
};

use super::{OrdrFuture, Request};

enum ReplaySource<'a> {
    File(&'a [u8]),
    Url(&'a str),
}

/// Commission a render job to o!rdr.
///
/// If successful, progress of the rendering can be tracking through the [`OrdrWebsocket`](crate::OrdrWebsocket).
#[must_use]
pub struct CommissionRender<'a> {
    ordr: &'a OrdrClient,
    replay_source: ReplaySource<'a>,
    username: &'a str,
    skin: &'a RenderSkinOption<'a>,
    options: Option<&'a RenderOptions>,
}

impl<'a> CommissionRender<'a> {
    pub(crate) const fn with_file(
        ordr: &'a OrdrClient,
        replay_file: &'a [u8],
        username: &'a str,
        skin: &'a RenderSkinOption<'a>,
    ) -> Self {
        Self {
            ordr,
            replay_source: ReplaySource::File(replay_file),
            username,
            skin,
            options: None,
        }
    }

    pub(crate) const fn with_url(
        ordr: &'a OrdrClient,
        replay_url: &'a str,
        username: &'a str,
        skin: &'a RenderSkinOption<'a>,
    ) -> Self {
        Self {
            ordr,
            replay_source: ReplaySource::Url(replay_url),
            username,
            skin,
            options: None,
        }
    }

    /// Specify rendering options.
    pub fn options(mut self, options: &'a RenderOptions) -> Self {
        self.options = Some(options);

        self
    }
}

impl IntoFuture for &mut CommissionRender<'_> {
    type Output = Result<RenderAdded, ClientError>;
    type IntoFuture = OrdrFuture<RenderAdded>;

    fn into_future(self) -> Self::IntoFuture {
        let missing_resolution = self.options.is_none();

        let mut form = self.options.map_or_else(Form::new, Form::serialize);

        if missing_resolution {
            form.push_text("resolution", RenderOptions::DEFAULT_RESOLUTION.as_str());
        }

        match self.replay_source {
            ReplaySource::File(bytes) => form.push_replay("replayFile", bytes),
            ReplaySource::Url(url) => form.push_text("replayURL", url),
        };

        form.push_text("username", self.username);

        match self.skin {
            RenderSkinOption::Official { name } => {
                form.push_text("skin", name.as_ref())
                    .push_text("customSkin", "false");
            }
            RenderSkinOption::Custom { id } => {
                form.push_text("skin", id.to_string())
                    .push_text("customSkin", "true");
            }
        }

        if let Some(verification) = self.ordr.verification() {
            form.push_text("verificationKey", verification.as_str());
        }

        self.ordr
            .request(Request::builder(Route::Render).form(form).build())
    }
}

impl IntoFuture for CommissionRender<'_> {
    type Output = Result<RenderAdded, ClientError>;
    type IntoFuture = OrdrFuture<RenderAdded>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
