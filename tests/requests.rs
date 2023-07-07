use std::time::Duration;

use rosu_render::{
    model::{RenderOptions, RenderSkinOption, Verification},
    websocket::event::RawEvent,
    OrdrClient, OrdrWebsocket,
};

#[tokio::test]
async fn test_render_success() {
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

#[tokio::test]
async fn test_custom_skin_error() {
    let client = OrdrClient::builder().build();

    let err = client.custom_skin_info(46).await.unwrap_err();
    println!("{err:#?}");
}
