use std::time::Duration;

use rosu_render::{
    model::{RenderOptions, RenderSkinOption, Verification},
    websocket::event::RawEvent,
    ClientError, OrdrClient, OrdrWebsocket,
};

#[tokio::test]
async fn render_success() {
    let replay_file = tokio::fs::read("./assets/2283307549.osr").await.unwrap();

    let mut websocket = OrdrWebsocket::connect().await.unwrap();

    let skin = RenderSkinOption::default();
    let settings = RenderOptions::default();

    let client = OrdrClient::builder()
        .verification(Verification::DevModeSuccess)
        .build();

    let render_added = client
        .render_with_replay_file(&replay_file, "rosu-render-success-test", &skin)
        .options(&settings)
        .await
        .unwrap();

    async fn await_render_done(websocket: &mut OrdrWebsocket, render_id: u32) {
        loop {
            match websocket.next_event().await {
                Ok(RawEvent::RenderDone(event)) if event.render_id == render_id => return,
                Ok(RawEvent::RenderProgress(event)) if event.render_id == render_id => {
                    let progress = event.deserialize().unwrap();
                    println!("{}: {}", progress.render_id, progress.progress);
                }
                Ok(RawEvent::RenderFailed(event)) if event.render_id == render_id => {
                    let failed = event.deserialize().unwrap();
                    panic!("Websocket error while awaiting render: {failed:?}");
                }
                Ok(_) => {}
                Err(err) => println!("Websocket error: {err:?}"),
            }
        }
    }

    let await_done_fut = await_render_done(&mut websocket, render_added.render_id);
    let timeout_res = tokio::time::timeout(Duration::from_secs(60), await_done_fut).await;
    timeout_res.unwrap_or_else(|_| panic!("Timed out while awaiting commissioned render"));

    websocket.disconnect().await.unwrap();
}

/// A nonexistent render ID must come back as a server error, which proves the
/// request path actually reached the API instead of failing in transport.
#[tokio::test]
async fn dyn_link_invalid_id() {
    let client = OrdrClient::builder().build();

    let err = client.dyn_link(1).await.unwrap_err();
    assert!(
        matches!(
            err,
            ClientError::Response { .. } | ClientError::Parsing { .. }
        ),
        "expected an API-level error, got: {err:#?}"
    );
}

#[tokio::test]
async fn bot_auth() {
    let key = match std::env::var("ORDR_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("skipping bot_auth: ORDR_KEY is not set");
            return;
        }
    };

    let mut websocket = OrdrWebsocket::connect().await.unwrap();

    match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        websocket.authenticate(&key),
    )
    .await
    {
        Ok(result) => result.unwrap(),
        Err(_) => panic!("Timed out awaiting bot_auth reply"),
    }

    websocket.disconnect().await.unwrap();
}

#[tokio::test]
async fn dyn_link() {
    let key = match std::env::var("ORDR_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("skipping dyn_link: ORDR_KEY is not set");
            return;
        }
    };
    let client = OrdrClient::builder()
        .verification(Verification::Key(key.into_boxed_str()))
        .build();

    let list = client.render_list().await.unwrap();

    // The newest render is often still rendering and has no dynlink yet.
    let render_id = list
        .renders
        .iter()
        .find(|r| r.progress.as_ref() == "Done.")
        .expect("no completed render in the global feed")
        .id;

    let link = client.dyn_link(render_id).await.unwrap();
    assert!(link.url.starts_with("http"));
}

#[tokio::test]
async fn custom_skin_error() {
    let client = OrdrClient::builder().build();

    let err = client.custom_skin_info(46).await.unwrap_err();
    println!("{err:#?}");
}
