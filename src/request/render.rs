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

    fn request(&self) -> Request {
        let missing_options = self.options.is_none();

        let mut form = self.options.map_or_else(Form::new, Form::serialize);

        if missing_options {
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

        Request::builder(Route::Render).form(form).build()
    }
}

impl IntoFuture for &mut CommissionRender<'_> {
    type Output = Result<RenderAdded, ClientError>;
    type IntoFuture = OrdrFuture<RenderAdded>;

    fn into_future(self) -> Self::IntoFuture {
        self.ordr.request(self.request())
    }
}

impl IntoFuture for CommissionRender<'_> {
    type Output = Result<RenderAdded, ClientError>;
    type IntoFuture = OrdrFuture<RenderAdded>;

    fn into_future(mut self) -> Self::IntoFuture {
        (&mut self).into_future()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        model::{RenderOptions, RenderSkinOption},
        OrdrClient,
    };

    use super::CommissionRender;

    fn form_body(render: &CommissionRender<'_>) -> String {
        let request = render.request();
        let form = request
            .form
            .expect("commission request must carry a multipart form");

        String::from_utf8(form.build()).expect("form body is UTF-8")
    }

    #[test]
    fn options_form() {
        let client = OrdrClient::builder().build();
        let skin = RenderSkinOption::Official {
            name: "Kuro".into(),
        };
        let options = RenderOptions::default();

        let render = client
            .render_with_replay_url("https://replay.watch/x.osr", "TestUser", &skin)
            .options(&options);

        let body = form_body(&render);

        assert!(body.contains(r#"name="replayURL""#));
        assert!(body.contains("https://replay.watch/x.osr"));
        assert!(body.contains(r#"name="username""#));
        assert!(body.contains("TestUser"));
        assert!(body.contains(r#"name="skin""#));
        assert!(body.contains("Kuro"));
        assert!(body.contains(r#"name="customSkin""#));
        assert!(body.contains(r#"name="resolution""#));
        assert!(body.contains("1280x720"));
        assert!(body.contains(r#"name="globalVolume""#));
        assert!(!body.contains(r#"name="verificationKey""#));
    }

    #[test]
    fn missing_options_default_resolution() {
        let client = OrdrClient::builder().build();
        let skin = RenderSkinOption::Official {
            name: "Kuro".into(),
        };

        let render = client.render_with_replay_url("https://replay.watch/x.osr", "TestUser", &skin);

        let body = form_body(&render);

        assert!(body.contains(r#"name="resolution""#));
        assert!(body.contains("1280x720"));
        assert!(!body.contains(r#"name="globalVolume""#));
    }

    #[test]
    fn custom_skin() {
        let client = OrdrClient::builder().build();
        let skin = RenderSkinOption::from(42u32);

        let render = client.render_with_replay_url("https://replay.watch/x.osr", "TestUser", &skin);

        let body = form_body(&render);
        let lines: Vec<&str> = body.lines().collect();

        let skin_idx = lines
            .iter()
            .position(|line| line.contains(r#"name="skin""#))
            .unwrap();
        let custom_idx = lines
            .iter()
            .position(|line| line.contains(r#"name="customSkin""#))
            .unwrap();

        assert_eq!(lines[skin_idx + 2], "42");
        assert_eq!(lines[custom_idx + 2], "true");
    }
}
