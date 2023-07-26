mod error;
mod track;
mod url;

use std::{collections::HashMap, fmt, net::SocketAddr, str::FromStr};

use age::x25519::Identity;
use axum::{
    extract::Query,
    response::Html,
    routing::{self, post},
    Router,
};
use axum_client_ip::SecureClientIpSource;
use clap::Parser;
use error::AppResult;
use eyre::{bail, Result};
use once_cell::sync::OnceCell;
use serde::{de, Deserialize, Deserializer, Serialize};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter};

use crate::error::AppError;
use crate::url::EncInput;
// use tracing_subscriber::

#[derive(Debug, Parser)]
struct Args {
    /// Address to bind to.
    #[arg(short, long, default_value = "0.0.0.0:8081")]
    address: String,

    /// URL base for creating links
    #[arg(long)]
    url: String,
}

static ID: OnceCell<Identity> = OnceCell::new();
static URL_BASE: OnceCell<String> = OnceCell::new();

const INDEX: &'static str = r#"
<!DOCTYPE html>
<html>

<body>
    <div style="text-align: center;">
        <h1>pixel-tracker</h1>
        <form>
            <label for="name">Tracking pixel name:</label><br>
            <input type="text" id="name" name="name" value="{{name}}" {{name_extra}}><br><br>

            <label for="webhook">Discord webhook URL:</label><br>
            <input type="text" id="webhook" name="webhook" value="{{webhook}}" {{webhook_extra}}><br><br>

            <input type="submit" value="Submit">
        </form>

        <h2><tt>{{result}}</tt></h2>
    </div>
</body>

</html>
"#;

#[tokio::main]
async fn main() -> Result<()> {
    {
        let layer_fmt = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .without_time()
            .with_line_number(true)
            .compact();

        let layer_error = tracing_error::ErrorLayer::default();

        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(layer_error)
            .with(layer_fmt)
            .init();

        color_eyre::install()?;
    }
    let args = Args::parse();

    ID.get_or_init(|| {
        let key = std::env::var("KEY").expect("Reading age KEY from environment");
        Identity::from_str(&key).expect("Parsing age key")
    });

    URL_BASE.set(args.url);

    let app = Router::new()
        .route("/", routing::get(router))
        .route("/pt/:encoded", routing::get(crate::track::tracker))
        .layer(SecureClientIpSource::ConnectInfo.into_extension())
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&args.address.parse()?)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

const TEMPL_NAME: &str = "index";

#[derive(Debug, Deserialize, Serialize)]
struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    name: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    webhook: Option<String>,
}

#[tracing::instrument(ret, err(Debug), level = "trace")]
async fn router(Query(params): Query<Params>) -> AppResult<Html<String>> {
    let mut handlebars = handlebars::Handlebars::new();
    handlebars.set_strict_mode(false);
    let mut vars: HashMap<&str, String> = HashMap::new();

    if let Params {
        name: Some(ref name),
        webhook: Some(ref webhook),
    } = params
    {
        vars.insert("name", name.clone());
        vars.insert("webhook", webhook.clone());

        let enc = crate::url::encrypt(
            EncInput {
                name: name.clone(),
                webhook: webhook.clone(),
            },
            vec![Box::new(ID.get().unwrap().to_public())],
        )?;

        let url = format!("{}/{}", URL_BASE.get().unwrap(), enc);
        vars.insert("result", url);
    }

    let render = handlebars.render_template(INDEX, &vars)?;

    Ok(Html(render))
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}
