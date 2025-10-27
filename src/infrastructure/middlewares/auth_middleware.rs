use actix_identity::Identity;
use actix_session::SessionExt;
use actix_web::{
    Error, FromRequest, HttpResponse, Result,
    body::{BoxBody, EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::header::LOCATION,
    middleware::Next,
};

pub async fn auth(
    mut req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<EitherBody<BoxBody>>, Error> {
    let session = req.get_session();
    let path = req.path().to_string();
    println!("Auth middleware triggered for path: {}", path);

    let (http_req, payload) = req.parts_mut();

    let identity = Identity::from_request(http_req, payload).await;

    if let Ok(id) = identity
        && id.id().is_ok()
    {
        let res = next.call(req).await?;

        return Ok(res.map_into_boxed_body().map_into_right_body());
    }

    session.insert("redirect_after_login", &path).ok();

    let response = HttpResponse::Found()
        .append_header((LOCATION, "/vibecall/auth/login"))
        .finish()
        .map_into_boxed_body();

    Ok(req.into_response(response).map_into_left_body())
}
