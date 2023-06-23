use std::future::IntoFuture;

use crate::{
    error::Error,
    model::{RenderCreated, RenderOptions, RenderSkinOption},
    routing::Route,
    util::multipart::Form,
    Ordr,
};

use super::{OrdrFuture, Request};

enum ReplaySource<'a> {
    File(&'a [u8]),
    Url(&'a str),
}

// TODO: docs
pub struct SendRender<'a> {
    ordr: &'a Ordr,
    replay_source: ReplaySource<'a>,
    username: &'a str,
    skin: &'a RenderSkinOption<'a>,
    options: Option<&'a RenderOptions>,
}

impl<'a> SendRender<'a> {
    pub(crate) const fn with_file(
        ordr: &'a Ordr,
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
        ordr: &'a Ordr,
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

    // TODO: docs
    pub fn options(mut self, options: &'a RenderOptions) -> Self {
        self.options = Some(options);

        self
    }
}

impl IntoFuture for &mut SendRender<'_> {
    type Output = Result<RenderCreated, Error>;
    type IntoFuture = OrdrFuture<RenderCreated>;

    fn into_future(self) -> Self::IntoFuture {
        let mut form = self.options.map_or_else(Form::new, Form::serialize);

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

impl IntoFuture for SendRender<'_> {
    type Output = Result<RenderCreated, Error>;
    type IntoFuture = OrdrFuture<RenderCreated>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}
