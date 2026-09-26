//! Commission an o!rdr render and follow it until it is done.
//!
//! The websocket is a global feed, so a supervisor task keeps the connection
//! alive and forwards only the events of our render. It reconnects when the
//! connection fails, and also after two minutes without an event to anticipate
//! dead connections.

use std::time::Duration;

use rosu_render::{
    model::{RenderSkinOption, Verification},
    websocket::event::RawEvent,
    OrdrClient, OrdrWebsocket,
};
use tokio::{
    sync::{mpsc, watch},
    time::{sleep, sleep_until, Instant},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // In production, use your key as verification instead.
    let client = OrdrClient::builder()
        .verification(Verification::DevModeSuccess)
        .render_ratelimit(10_000, 1, 1) // One render request per 10 seconds
        .build();

    let replay_file = tokio::fs::read("assets/2283307549.osr").await?;
    let render = client
        .render_with_replay_file(
            &replay_file,
            "rosu-render-example",
            &RenderSkinOption::default(),
        )
        .await?;

    let render_id = render.render_id;
    println!("Render {render_id} is queued.");

    // The supervisor keeps the websocket connected and forwards the events
    // for this render.
    let (events_tx, mut events_rx) = mpsc::unbounded_channel();
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let supervisor = tokio::spawn(supervisor(render_id, events_tx, shutdown_rx));

    // Follow the render until it is done or fails.
    let video_url = loop {
        let Some(event) = events_rx.recv().await else {
            return Err("supervisor shut down unexpectedly".into());
        };

        match event {
            RawEvent::RenderProgress(progress) => println!("Progress: {progress:?}"),
            RawEvent::RenderDone(done) => break done.deserialize()?.video_url,
            RawEvent::RenderFailed(failed) => {
                return Err(format!("render failed: {failed:?}").into());
            }
            _ => {}
        }
    };

    println!("Render done: {video_url}");

    // Shut the supervisor down.
    shutdown_tx.send(()).ok();
    supervisor.await?;

    Ok(())
}

async fn supervisor(
    render_id: u32,
    events: mpsc::UnboundedSender<RawEvent>,
    mut shutdown: watch::Receiver<()>,
) {
    const IDLE_TIMEOUT: Duration = Duration::from_secs(2 * 60);

    loop {
        if shutdown.changed().await.is_err() {
            return;
        }

        let mut websocket = match OrdrWebsocket::connect().await {
            Ok(websocket) => websocket,
            Err(err) => {
                eprintln!("Failed to connect: {err}; retrying in a second");
                sleep(Duration::from_secs(1)).await;

                continue;
            }
        };

        let mut idle = Instant::now() + IDLE_TIMEOUT;

        loop {
            tokio::select! {
                _ = shutdown.changed() => return,
                _ = sleep_until(idle) => {
                    eprintln!("No events for two minutes; reconnecting");

                    break;
                }
                result = websocket.next_event() => {
                    idle = Instant::now() + IDLE_TIMEOUT;

                    match result {
                        Ok(event) => {
                            let id = match &event {
                                RawEvent::RenderProgress(progress) => progress.render_id,
                                RawEvent::RenderDone(done) => done.render_id,
                                RawEvent::RenderFailed(failed) => failed.render_id,
                                _ => continue,
                            };

                            if id == render_id && events.send(event).is_err() {
                                // The receiver is gone, i.e. main is done.
                                return;
                            }
                        }
                        Err(err) => {
                            eprintln!("Websocket error: {err}; reconnecting");

                            break;
                        }
                    }
                }
            }
        }
    }
}
