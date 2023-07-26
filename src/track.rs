use axum::{
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
};
use axum_client_ip::{InsecureClientIp, SecureClientIp};
use serenity::{
    http::Http,
    model::{prelude::Embed, webhook::Webhook},
};
use tracing::{info, instrument};

use crate::error::AppResult;

static PIXEL: &[u8] = include_bytes!("../1x1.png");

#[instrument(ret, err(Debug), level = "info")]
pub async fn tracker(
    Path(encoded): Path<String>,
    ip: SecureClientIp,
) -> AppResult<impl IntoResponse> {
    let id = crate::ID.get().unwrap();

    let result = crate::url::decrypt(encoded, &id)?;
    info!(?result);

    let http = Http::new("");

    let webhook = Webhook::from_url(&http, &result.webhook).await?;

    let now = time::OffsetDateTime::now_utc();

    webhook
        .execute(&http, true, |w| {
            w.embeds(vec![Embed::fake(|e| {
                e.title(format!("Tracking read @ {} (UTC)", now))
                    .field("Name", &result.name, false)
                    .field("IP", ip.0, false)
            })])
        })
        .await?;

    Ok((
        // -
        [
            // -
            (header::CONTENT_TYPE, "image/png"),
        ],
        PIXEL,
    ))
}
