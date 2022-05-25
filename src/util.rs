use std::{future::Future, task::Poll};

use axum::http::{header, uri::PathAndQuery, HeaderValue, Request, Response, StatusCode, Uri};
use pin_project_lite::pin_project;
use serde::{de, Deserialize, Deserializer};
use tower::Service;

/// Serde deserialization decorator to map empty Strings to `true`.
pub(crate) fn empty_string_as_true<'de, D>(de: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(true),
        Some(s) => s.parse().map_err(de::Error::custom),
    }
}

/// Redirects `p.z.cash` to `https://protocol.z.cash`.
#[derive(Clone)]
pub(crate) struct RedirectDomain<S> {
    inner: S,
}

impl<S> RedirectDomain<S> {
    pub(crate) fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RedirectDomain<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = RedirectFuture<S::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        if req.headers().get(header::HOST) == Some(&HeaderValue::from_static("p.z.cash")) {
            let location = Uri::builder()
                .scheme("https")
                .authority("protocol.z.cash")
                .path_and_query(
                    req.uri()
                        .path_and_query()
                        .cloned()
                        .unwrap_or_else(|| PathAndQuery::from_static("")),
                )
                .build()
                .unwrap();

            RedirectFuture {
                inner: Kind::Redirect {
                    status_code: StatusCode::TEMPORARY_REDIRECT,
                    location: Some(
                        HeaderValue::try_from(location.to_string())
                            .expect("URI isn't a valid header value"),
                    ),
                },
            }
        } else {
            RedirectFuture {
                inner: Kind::Serve {
                    future: self.inner.call(req),
                },
            }
        }
    }
}

pin_project! {
    /// Response future for [`RedirectDomain`].
    pub(crate) struct RedirectFuture<F> {
        #[pin]
        inner: Kind<F>,
    }
}

pin_project! {
    #[project = KindProj]
    enum Kind<F> {
        Redirect {
            status_code: StatusCode,
            location: Option<HeaderValue>,
        },
        Serve {
            #[pin]
            future: F,
        }
    }
}

impl<F, B, E> Future for RedirectFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Default,
{
    type Output = Result<Response<B>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.project() {
            KindProj::Redirect {
                status_code,
                location,
            } => {
                let mut response = Response::new(B::default());

                *response.status_mut() = *status_code;

                response
                    .headers_mut()
                    .insert(header::LOCATION, location.take().unwrap());

                Poll::Ready(Ok(response))
            }
            KindProj::Serve { future } => future.poll(cx),
        }
    }
}
