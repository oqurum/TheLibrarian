use std::{
    future::{ready, Ready},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_identity::Identity;
use actix_web::{
    body::MessageBody,
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform, Extensions},
    FromRequest, HttpRequest,
};
use chrono::Utc;
use common::{api::ApiErrorResponse, MemberId};
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

use crate::{model::MemberModel, InternalError, Result, WebError};

pub mod external;
pub mod password;
pub mod passwordless;

#[derive(Serialize, Deserialize)]
pub struct CookieAuth {
    pub member_id: MemberId,
    pub stored_since: i64,
}

pub fn get_auth_value(identity: &Identity) -> Option<CookieAuth> {
    let ident = identity.id().ok()?;
    serde_json::from_str(&ident).ok()
}

pub fn remember_member_auth(ext: &Extensions, member_id: MemberId) -> Result<()> {
    let value = serde_json::to_string(&CookieAuth {
        member_id,
        stored_since: Utc::now().timestamp_millis(),
    })?;

    Identity::login(ext, value).expect("Ident Login Error");

    Ok(())
}

// Retrive Member from Identity
pub struct MemberCookie(CookieAuth);

impl MemberCookie {
    pub fn member_id(&self) -> MemberId {
        self.0.member_id
    }

    pub async fn fetch(&self, client: &Client) -> Result<Option<MemberModel>> {
        MemberModel::get_by_id(self.member_id(), client).await
    }

    pub async fn fetch_or_error(&self, client: &Client) -> Result<MemberModel> {
        match self.fetch(client).await? {
            Some(v) => Ok(v),
            None => Err(InternalError::UserMissing.into()),
        }
    }
}

impl FromRequest for MemberCookie {
    type Error = WebError;
    type Future =
        Pin<Box<dyn std::future::Future<Output = std::result::Result<MemberCookie, WebError>>>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let fut = Identity::from_request(req, pl);

        Box::pin(async move {
            if let Some(id) = get_auth_value(&fut.await?) {
                Ok(MemberCookie(id))
            } else {
                Err(WebError::ApiResponse(ApiErrorResponse::new("unauthorized")))
            }
        })
    }
}

pub struct LoginRequired;

impl<S, B> Transform<S, ServiceRequest> for LoginRequired
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = WebError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = WebError;
    type Transform = CheckLoginMiddleware<S>;
    type InitError = ();
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckLoginMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct CheckLoginMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CheckLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = WebError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = WebError;
    type Future = LocalBoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);

        async move {
            let (r, mut pl) = req.into_parts();

            let identity = Identity::from_request(&r, &mut pl).await?;

            if get_auth_value(&identity).is_some() {
                // HttpRequest contains an Rc so we need to drop identity to free the cloned one.
                drop(identity);

                Ok(srv.call(ServiceRequest::from_parts(r, pl)).await?)
            } else {
                Err(WebError::ApiResponse(ApiErrorResponse::new("unauthorized")))
            }
        }
        .boxed_local()
    }
}
