use std::cell::RefCell;
use std::future::{Ready, ready};
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Instant;

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage, web};
use futures_util::future::LocalBoxFuture;
use tracing::info;
use uuid::Uuid;

use crate::application::auth_service::AuthService;
use crate::infrastructure::jwt::JwtKeys;
use crate::presentation::auth::extract_user_from_token;

static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");
static TIMING_HEADER: HeaderName = HeaderName::from_static("server-timing");

#[derive(Clone)]
pub struct RequestId(pub String);

pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdService { service }))
    }
}

pub struct RequestIdService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = req
            .headers()
            .get(&REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        req.extensions_mut().insert(RequestId(request_id.clone()));

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            res.response_mut().headers_mut().insert(
                REQUEST_ID_HEADER.clone(),
                HeaderValue::from_str(&request_id).unwrap(),
            );
            Ok(res)
        })
    }
}

pub struct TimingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TimingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TimingService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TimingService { service }))
    }
}

pub struct TimingService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TimingService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_owned();
        let rid = req.extensions().get::<RequestId>().map(|id| id.0.clone());

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let duration = start.elapsed();
            let status = res.status().as_u16();
            if let Some(rid) = rid {
                info!(
                    request_id = %rid,
                    method = %method,
                    path = %path,
                    status,
                    duration_ms = duration.as_millis(),
                    "request completed"
                );
            } else {
                info!(
                    method = %method,
                    path = %path,
                    status,
                    duration_ms = duration.as_millis(),
                    "request completed"
                );
            }

            if let Ok(value) = HeaderValue::from_str(&format!("app;dur={}", duration.as_millis())) {
                res.response_mut()
                    .headers_mut()
                    .insert(TIMING_HEADER.clone(), value);
            }

            Ok(res)
        })
    }
}

pub struct JwtAuthMiddleware {
    keys: JwtKeys,
}

impl JwtAuthMiddleware {
    pub fn new(keys: JwtKeys) -> Self {
        Self { keys }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthService {
            service: Rc::new(RefCell::new(service)),
            keys: self.keys.clone(),
        }))
    }
}

pub struct JwtAuthService<S> {
    service: Rc<RefCell<S>>,
    keys: JwtKeys,
}

impl<S, B> Service<ServiceRequest> for JwtAuthService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let keys = self.keys.clone();
        let service = Rc::clone(&self.service);
        let auth_service = req.app_data::<web::Data<AuthService>>().cloned();

        let auth_header = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        Box::pin(async move {
            // If auth header is present, validate and inject user.
            // If absent, pass through — handler's AuthenticatedUser extractor will reject if needed.
            if let Some(header) = auth_header {
                if let Some(token) = header.strip_prefix("Bearer ") {
                    if let Some(auth_service) = auth_service {
                        if let Ok(user) =
                            extract_user_from_token(token, &keys, auth_service.get_ref()).await
                        {
                            req.extensions_mut().insert(user);
                        }
                    }
                }
            }

            let fut = {
                let svc = service.borrow_mut();
                svc.call(req)
            };

            let res = fut.await?;
            Ok(res)
        })
    }
}
