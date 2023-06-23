use std::{
    future::Future,
    marker::PhantomData,
    pin::{pin, Pin},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use hyper::{
    body::{self, Bytes},
    client::ResponseFuture as HyperResponseFuture,
    StatusCode,
};
use leaky_bucket::AcquireOwned;
use pin_project::pin_project;
use serde::de::DeserializeOwned;

use crate::error::Error;

use super::requestable::Requestable;

#[pin_project(project = OrdrFutureProj)]
pub struct OrdrFuture<T> {
    #[pin]
    ratelimit: Option<AcquireOwned>,
    #[pin]
    state: OrdrFutureState<T>,
}

impl<T> OrdrFuture<T> {
    pub(crate) const fn new(
        fut: Pin<Box<HyperResponseFuture>>,
        banned: Arc<AtomicBool>,
        ratelimit: AcquireOwned,
    ) -> Self {
        Self {
            ratelimit: Some(ratelimit),
            state: OrdrFutureState::InFlight(InFlight {
                fut,
                banned,
                phantom: PhantomData,
            }),
        }
    }

    pub(crate) const fn error(source: Error) -> Self {
        Self {
            ratelimit: None,
            state: OrdrFutureState::Failed(Some(source)),
        }
    }

    fn await_ratelimit(
        mut ratelimit_opt: Pin<&mut Option<AcquireOwned>>,
        cx: &mut Context,
    ) -> Poll<()> {
        if let Some(ratelimit) = ratelimit_opt.as_mut().as_pin_mut() {
            match ratelimit.poll(cx) {
                Poll::Ready(_) => ratelimit_opt.set(None),
                Poll::Pending => return Poll::Pending,
            }
        }

        Poll::Ready(())
    }
}

impl<T: DeserializeOwned + Requestable> Future for OrdrFuture<T> {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let mut state = this.state.as_mut();

        match state.as_mut().project() {
            OrdrFutureStateProj::InFlight(in_flight) => {
                if Self::await_ratelimit(this.ratelimit, cx).is_pending() {
                    return Poll::Pending;
                }

                match in_flight.poll(cx) {
                    Poll::Ready(Ok(chunking)) => {
                        state.set(OrdrFutureState::Chunking(chunking));
                        cx.waker().wake_by_ref();

                        Poll::Pending
                    }
                    Poll::Ready(Err(err)) => {
                        state.set(OrdrFutureState::Completed);

                        Poll::Ready(Err(err))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            OrdrFutureStateProj::Chunking(chunking) => match chunking.poll(cx) {
                Poll::Ready(res) => {
                    state.set(OrdrFutureState::Completed);

                    Poll::Ready(res)
                }
                Poll::Pending => Poll::Pending,
            },
            OrdrFutureStateProj::Failed(failed) => {
                let err = failed.take().expect("error already taken");
                state.set(OrdrFutureState::Completed);

                Poll::Ready(Err(err))
            }
            OrdrFutureStateProj::Completed => panic!("future already completed"),
        }
    }
}

#[pin_project(project = OrdrFutureStateProj)]
enum OrdrFutureState<T> {
    Chunking(#[pin] Chunking<T>),
    Completed,
    Failed(Option<Error>),
    InFlight(#[pin] InFlight<T>),
}

#[pin_project]
struct Chunking<T> {
    #[pin]
    fut: Pin<Box<dyn Future<Output = Result<Bytes, Error>> + Send + Sync + 'static>>,
    status: StatusCode,
    phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Requestable> Future for Chunking<T> {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let bytes = match this.fut.poll(cx) {
            Poll::Ready(Ok(bytes)) => bytes,
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            Poll::Pending => return Poll::Pending,
        };

        let res = if this.status.is_success() {
            match serde_json::from_slice(&bytes) {
                Ok(this) => Ok(this),
                Err(source) => Err(Error::Parsing {
                    body: bytes.into(),
                    source,
                }),
            }
        } else {
            Err(<T as Requestable>::response_error(*this.status, bytes))
        };

        Poll::Ready(res)
    }
}

#[pin_project]
struct InFlight<T> {
    #[pin]
    fut: Pin<Box<HyperResponseFuture>>,
    banned: Arc<AtomicBool>,
    phantom: PhantomData<T>,
}

impl<T: Requestable> Future for InFlight<T> {
    type Output = Result<Chunking<T>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let response = match this.fut.poll(cx) {
            Poll::Ready(Ok(response)) => response,
            Poll::Ready(Err(source)) => return Poll::Ready(Err(Error::RequestError { source })),
            Poll::Pending => return Poll::Pending,
        };

        let status = response.status();

        match status {
            StatusCode::TOO_MANY_REQUESTS => warn!("429 response: {response:?}"),
            StatusCode::UNAUTHORIZED => this.banned.store(true, Ordering::Relaxed),
            StatusCode::SERVICE_UNAVAILABLE => {
                return Poll::Ready(Err(Error::ServiceUnavailable { response }))
            }
            _ => {}
        };

        // body::to_bytes returns an anonymous future so we need to Box::pin it
        let fut = async {
            let body = response.into_body();

            body::to_bytes(body)
                .await
                .map_err(|source| Error::ChunkingResponse { source })
        };

        Poll::Ready(Ok(Chunking {
            fut: Box::pin(fut),
            status,
            phantom: PhantomData,
        }))
    }
}
