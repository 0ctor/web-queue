use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::utils::response::ApiResponse;

pub struct RequestConcurrencyLimiter {
    semaphore: Option<Arc<Semaphore>>,
}

impl RequestConcurrencyLimiter {
    pub fn new(max_in_flight: usize) -> Self {
        let semaphore = if max_in_flight == 0 {
            None
        } else {
            Some(Arc::new(Semaphore::new(max_in_flight)))
        };
        Self { semaphore }
    }
}

fn is_exempt_path(path: &str) -> bool {
    path == "/health/live"
        || path == "/health/ready"
        || path == "/v2/health"
        || path == "/v2/health/ready"
        || path.starts_with("/storage/")
}

impl<S, B> Transform<S, ServiceRequest> for RequestConcurrencyLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = actix_web::Error;
    type Transform = RequestConcurrencyLimiterService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestConcurrencyLimiterService {
            service: Rc::new(service),
            semaphore: self.semaphore.clone(),
        }))
    }
}

pub struct RequestConcurrencyLimiterService<S> {
    service: Rc<S>,
    semaphore: Option<Arc<Semaphore>>,
}

impl<S, B> Service<ServiceRequest> for RequestConcurrencyLimiterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let service = self.service.clone();
        let semaphore = self.semaphore.clone();

        Box::pin(async move {
            let _permit = if let Some(sem) = semaphore {
                if is_exempt_path(&path) {
                    None
                } else {
                    match sem.clone().try_acquire_owned() {
                        Ok(p) => Some(p),
                        Err(_) => {
                            let res = req.into_response(
                                HttpResponse::ServiceUnavailable()
                                    .json(ApiResponse::<()>::error(
                                        "Too many concurrent requests".to_string(),
                                        "load shed".to_string(),
                                    ))
                                    .map_into_boxed_body(),
                            );
                            return Ok(res);
                        }
                    }
                }
            } else {
                None
            };

            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}
