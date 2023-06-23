use std::{collections::HashMap, ops::Deref, sync::Arc, time::Duration};

use rosu_render::{
    client::OrdrBuilder,
    model::{
        RenderDone, RenderFail, RenderOptions, RenderProgress, RenderSkinOption, Verification,
    },
    Ordr,
};
use tokio::sync::{mpsc, RwLock};

struct Client {
    ordr: Ordr,
    senders: Arc<RwLock<HashMap<u32, Senders>>>,
}

impl Client {
    async fn new(ordr: OrdrBuilder) -> Self {
        let senders = Arc::new(RwLock::new(HashMap::<u32, Senders>::new()));

        let done_clone = Arc::clone(&senders);
        let fail_clone = Arc::clone(&senders);
        let progress_clone = Arc::clone(&senders);

        let ordr = ordr
            .on_render_done(move |msg| {
                let done_clone = Arc::clone(&done_clone);

                Box::pin(async move {
                    let render_id = msg.render_id;
                    let guard = done_clone.read().await;

                    if let Some(senders) = guard.get(&render_id) {
                        let _ = senders.done.send(msg).await;
                    }
                })
            })
            .on_render_failed(move |msg| {
                let fail_clone = Arc::clone(&fail_clone);

                Box::pin(async move {
                    let render_id = msg.render_id;
                    let guard = fail_clone.read().await;

                    if let Some(senders) = guard.get(&render_id) {
                        let _ = senders.failed.send(msg).await;
                    }
                })
            })
            .on_render_progress(move |msg| {
                let progress_clone = Arc::clone(&progress_clone);

                Box::pin(async move {
                    let render_id = msg.render_id;
                    let guard = progress_clone.read().await;

                    if let Some(senders) = guard.get(&render_id) {
                        let _ = senders.progress.send(msg).await;
                    }
                })
            })
            .build()
            .await
            .unwrap();

        Self { ordr, senders }
    }

    async fn subscribe_render_id(&self, render_id: u32) -> Receivers {
        let (done_tx, done_rx) = mpsc::channel(1);
        let (failed_tx, failed_rx) = mpsc::channel(1);
        let (progress_tx, progress_rx) = mpsc::channel(8);

        let senders = Senders {
            done: done_tx,
            failed: failed_tx,
            progress: progress_tx,
        };

        let receivers = Receivers {
            done: done_rx,
            failed: failed_rx,
            progress: progress_rx,
        };

        self.senders.write().await.insert(render_id, senders);

        receivers
    }

    async fn unsubscribe_render_id(&self, render_id: u32) {
        self.senders.write().await.remove(&render_id);
    }
}

impl Deref for Client {
    type Target = Ordr;

    fn deref(&self) -> &Self::Target {
        &self.ordr
    }
}

struct Senders {
    done: mpsc::Sender<RenderDone>,
    failed: mpsc::Sender<RenderFail>,
    progress: mpsc::Sender<RenderProgress>,
}

struct Receivers {
    done: mpsc::Receiver<RenderDone>,
    failed: mpsc::Receiver<RenderFail>,
    #[allow(unused)]
    progress: mpsc::Receiver<RenderProgress>,
}

#[tokio::test]
async fn test_render_success() {
    let replay_file = tokio::fs::read("./assets/replay-osu_658127_2283307549.osr")
        .await
        .unwrap();

    let skin = RenderSkinOption::default();
    let settings = RenderOptions::default();

    let builder = Ordr::builder()
        .with_websocket(true)
        .verification(Verification::DevModeSuccess);
    let ordr = Client::new(builder).await;

    let render_created = ordr
        .render_with_replay_file(&replay_file, "rosu-render-success-test", &skin)
        .options(&settings)
        .await
        .unwrap();

    let mut receivers = ordr.subscribe_render_id(render_created.render_id).await;

    let await_done = async {
        loop {
            tokio::select! {
                _ = receivers.done.recv() => break,
                failed = receivers.failed.recv() => panic!("Websocket error while awaiting render: {failed:?}"),
                progress = receivers.progress.recv() => {
                    let progress = progress.unwrap();
                    println!("{}: {}", progress.render_id, progress.progress);
                }
            }
        }
    };

    let timeout_res = tokio::time::timeout(Duration::from_secs(60), await_done).await;
    ordr.unsubscribe_render_id(render_created.render_id).await;
    timeout_res.unwrap_or_else(|_| panic!("Timed out while awaiting commissioned render"));
}

#[tokio::test]
async fn test_custom_skin_error() {
    let builder = Ordr::builder().with_websocket(false);
    let ordr = Client::new(builder).await;

    let err = ordr.custom_skin_info(46).await.unwrap_err();
    println!("{err:#?}");
}
