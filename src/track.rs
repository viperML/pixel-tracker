use axum::{
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
};
use axum_client_ip::XForwardedFor;
use eyre::ContextCompat;
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
    XForwardedFor(ips): XForwardedFor,
    headers: HeaderMap,
) -> AppResult<impl IntoResponse> {
    let id = &crate::CONFIG.args.key;

    let result = crate::transport::decrypt(encoded, &id)?;
    info!(?result);

    let http = Http::new("");

    let webhook = Webhook::from_url(&http, &result.webhook).await?;

    let now = time::OffsetDateTime::now_utc();

    let ip = ips.first().wrap_err("Couldn't get first IP")?;

    webhook
        .execute(&http, true, |w| {
            w.embeds(vec![Embed::fake(|e| {
                e.title(format!("Tracking read @ {} (UTC)", now))
                    .field("Name", &result.name, false)
                    .field("IP", ip, false)
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
