//! In this example we create a wrapper type that contains an [`OrdrClient`]
//! and also handles events from the [`OrdrWebsocket`] specifically by subscribing
//! to certain render ids and only forwarding interesting events of those ids.

use std::{collections::HashMap, error::Error as StdError, sync::Arc};

use rosu_render::{
    model::{RenderDone, RenderProgress, RenderSkinOption, Verification},
    websocket::event::RawEvent,
    OrdrClient, OrdrWebsocket, WebsocketError,
};
use tokio::{
    sync::{mpsc, oneshot, RwLock},
    task::JoinHandle,
};

// Passing events from the websocket will happen through mpsc channels.
// The websocket will put events into these senders.
struct OrdrSenders {
    // In this example we're only interested in `done` and `progress` events.
    done: mpsc::Sender<RenderDone>,
    progress: mpsc::Sender<RenderProgress>,
}

// And these receivers will then receive the websocket events.
pub struct OrdrReceivers {
    pub done: mpsc::Receiver<RenderDone>,
    pub progress: mpsc::Receiver<RenderProgress>,
}

type Senders = Arc<RwLock<HashMap<u32, OrdrSenders>>>;

// Our wrapping struct to handle all o!rdr things.
pub struct Ordr {
    // The o!rdr client to communicate with the API
    client: OrdrClient,
    // These senders will allow us to (un)subscribe to specific render ids.
    senders: Senders,
    // When we're done with everything, we should let the websocket know that it should disconnect
    shutdown_tx: oneshot::Sender<()>,
    // This handle to the websocket's worker thread will let us await a successful disconnect
    websocket_handle: JoinHandle<()>,
}

impl Ordr {
    // To subscribe to a render id we:
    //   - create a new mpsc channel
    //   - put its sender into our `senders` map for the given render id
    //   - then return the receiver
    pub async fn subscribe_render_id(&self, render_id: u32) -> OrdrReceivers {
        let (done_tx, done_rx) = mpsc::channel(1);
        let (progress_tx, progress_rx) = mpsc::channel(1);

        let senders = OrdrSenders {
            done: done_tx,
            progress: progress_tx,
        };

        let receivers = OrdrReceivers {
            done: done_rx,
            progress: progress_rx,
        };

        self.senders.write().await.insert(render_id, senders);

        receivers
    }

    // To unsubscribe, we simple remove the render id entry in our `senders` map
    pub async fn unsubscribe_render_id(&self, render_id: u32) {
        self.senders.write().await.remove(&render_id);
    }

    pub async fn new() -> Result<Self, WebsocketError> {
        // First connect to the websocket and create the client
        let websocket = OrdrWebsocket::connect().await?;

        // When running on debug, we should only *simulate* render commissions
        // so we don't spam the real thing while we're just testing
        #[cfg(debug_assertions)]
        let verification = Verification::DevModeSuccess;

        // On release we will use our proper verification key instead
        #[cfg(not(debug_assertions))]
        let verification = Verification::Key("my_verification_key".into());

        let client = OrdrClient::builder().verification(verification).build();

        // Then create an empty map of senders
        let senders = Senders::default();

        // This oneshot channel lets us notify the websocket when to disconnect
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Now we move the websocket into its own worker thread to do its thing there
        let websocket_handle = tokio::spawn(Self::handle_websocket_events(
            websocket,
            Arc::clone(&senders),
            shutdown_rx,
        ));

        Ok(Self {
            client,
            senders,
            shutdown_tx,
            websocket_handle,
        })
    }

    async fn handle_websocket_events(
        mut websocket: OrdrWebsocket,
        senders: Senders,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) {
        loop {
            // Either receive the next event or get notified of a shutdown
            let event_result = tokio::select! {
                event_result = websocket.next_event() => event_result,
                _ = &mut shutdown_rx => match websocket.disconnect().await {
                    Ok(_) => return,
                    Err(err) => {
                        println!("Failed to disconnect websocket gracefully: {err}");

                        return
                    }
                },
            };

            // Keep awaiting the next websocket event
            match event_result {
                // We're only interested in `done` and `progress` events so only check for those
                Ok(event) => match event {
                    RawEvent::RenderDone(event) => {
                        let guard = senders.read().await;

                        // Check if the event's render id is of interest
                        if let Some(senders) = guard.get(&event.render_id) {
                            // If so, deserialize the event and forward it into the channel
                            match event.deserialize() {
                                Ok(done) => {
                                    let _ = senders.done.send(done).await;
                                }
                                Err(err) => println!("Failed to deserialize RenderDone: {err}"),
                            }
                        }
                    }
                    // And do the same thing for `progress` events
                    RawEvent::RenderProgress(event) => {
                        let guard = senders.read().await;

                        if let Some(senders) = guard.get(&event.render_id) {
                            match event.deserialize() {
                                Ok(progress) => {
                                    let _ = senders.progress.send(progress).await;
                                }
                                Err(err) => {
                                    println!("Failed to deserialize RenderProgress: {err}")
                                }
                            }
                        }
                    }
                    _ => {} // the other events are ignored in this example
                },
                Err(err) => println!("Websocket error: {err}"),
            }
        }
    }

    // Once we're done with everything, we can shutdown
    // gracefully by disconnecting from the websocket
    pub async fn disconnect(self) {
        if self.shutdown_tx.send(()).is_ok() {
            self.websocket_handle
                .await
                .expect("websocket worker panicked");
        }
    }
}

// Now let's use all this
#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    // First we create our wrapper
    let ordr = Ordr::new().await?;

    // Then we commission a render
    let replay = tokio::fs::read("./assets/2283307549.osr").await?;
    let skin = RenderSkinOption::default();

    let commission = ordr
        .client
        .render_with_replay_file(&replay, "rosu-render-example", &skin)
        .await?;

    // Then we subscribe to its render id
    let mut receivers = ordr.subscribe_render_id(commission.render_id).await;

    // And now we listen to events until the render is done
    loop {
        tokio::select! {
            // RenderProgress event received, print the update
            event = receivers.progress.recv() => {
                let event = event.expect("sender was dropped");
                println!("Progress: {}", event.progress);
            }
            // RenderDone event received, print the video url and break the loop
            event = receivers.done.recv() => {
                let event = event.expect("sender was dropped");
                println!("Done: URL={}", event.video_url);

                break;
            }
        }

        // Note that we currently only break out of this loop on a RenderDone event
        // so if the render failed with a RenderFailed event we wouldn't notice
        // and keep looping. If you stick to this example, you probably also
        // want to listen to RenderFailed events.
    }

    // Not necessary to unsubscribe in this example but let's do it anyway
    ordr.unsubscribe_render_id(commission.render_id).await;

    // We're done, let's disconnect gracefully
    ordr.disconnect().await;

    Ok(())
}
