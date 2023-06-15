use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc, OnceLock,
    },
    time::Duration,
};

use rosu_render::{
    model::{
        RenderDone, RenderFail, RenderOptions, RenderProgress, RenderSkinOption, Verification,
    },
    Ordr,
};
use tokio::sync::{mpsc, Mutex, MutexGuard, RwLock};

static ORDR: Client = Client {
    initializing: AtomicBool::new(false),
    initialized: AtomicBool::new(false),
    inner: OnceLock::new(),
    senders: OnceLock::new(),
};

struct Client {
    initializing: AtomicBool,
    initialized: AtomicBool,
    inner: OnceLock<Mutex<Ordr>>,
    senders: OnceLock<Arc<RwLock<HashMap<u32, Senders>>>>,
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

impl Client {
    async fn get() -> MutexGuard<'static, Ordr> {
        let initializing = ORDR.initialized.load(SeqCst);
        let initialized = ORDR.initialized.load(SeqCst);

        match (initializing, initialized) {
            (false, false) => {
                match ORDR
                    .initializing
                    .compare_exchange(false, true, SeqCst, SeqCst)
                {
                    Ok(_) => {
                        let senders = Arc::new(RwLock::new(HashMap::<u32, Senders>::new()));

                        let done_clone = Arc::clone(&senders);
                        let fail_clone = Arc::clone(&senders);
                        let progress_clone = Arc::clone(&senders);

                        let client = Ordr::builder()
                            .with_websocket(true)
                            .verification(Verification::DevModeSuccess)
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

                        if ORDR.inner.set(Mutex::new(client)).is_err() {
                            panic!("Client has already been set");
                        }

                        if ORDR.senders.set(senders).is_err() {
                            panic!("Senders have already been set");
                        }

                        let _ = ORDR
                            .initialized
                            .compare_exchange(false, true, SeqCst, SeqCst);
                        let _ = ORDR
                            .initializing
                            .compare_exchange(true, false, SeqCst, SeqCst);
                    }
                    Err(_) => Self::block_until_initialized(),
                }
            }
            (true, false) => Self::block_until_initialized(),
            (false, true) | (true, true) => {}
        }

        ORDR.inner.get().unwrap().lock().await
    }

    fn block_until_initialized() {
        let mut old = true;

        loop {
            match ORDR
                .initialized
                .compare_exchange_weak(old, true, SeqCst, SeqCst)
            {
                Ok(_) => break,
                Err(b) => old = b,
            }
        }
    }

    async fn subscribe_render_id(render_id: u32) -> Receivers {
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

        ORDR.senders
            .get()
            .unwrap()
            .write()
            .await
            .insert(render_id, senders);

        receivers
    }

    async fn unsubscribe_render_id(render_id: u32) {
        ORDR.senders.get().unwrap().write().await.remove(&render_id);
    }
}

#[tokio::test]
async fn test_render() {
    let replay_file = tokio::fs::read("./assets/replay-osu_658127_2283307549.osr")
        .await
        .unwrap();

    let skin = RenderSkinOption::new("Danser default skin (Redd glass)", true);
    let settings = RenderOptions::default();

    let render_created = Client::get()
        .await
        .render_with_replay_file(&replay_file, "rosu-render-test", &skin)
        .options(&settings)
        .await
        .unwrap();

    let mut receivers = Client::subscribe_render_id(render_created.render_id).await;

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

    let timeout_res = tokio::time::timeout(Duration::from_secs(45), await_done).await;
    Client::unsubscribe_render_id(render_created.render_id).await;
    timeout_res.unwrap_or_else(|_| panic!("Timed out while awaiting commissioned render"));
}
