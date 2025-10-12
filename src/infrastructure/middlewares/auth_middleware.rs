use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

pub async fn auth(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    let path = req.path();
    println!("Request path: {}", path);

    // invoke the wrapped middleware or service
    let res = next.call(req).await?;

    // post-processing

    Ok(res)
}
