use anyhow::Result;
use env_logger::{Builder, Env, Target};
use http::Uri;
use log::{error, info};
use rustls::{client::ServerCertVerified, Certificate, Error, ServerName};
use serde::{Deserialize, Deserializer};
use std::{
    ffi::OsString, fs::File, io::Read, net::SocketAddr, path::Path, sync::Arc, time::SystemTime,
};
use tokio::net::TcpListener;

use xmpp_proxy::{
    common::{
        certs_key::CertsKey,
        outgoing::{OutgoingConfig, OutgoingVerifierConfig},
        shuffle_rd_wr_filter_only,
    },
    context::Context,
    in_out::{StanzaRead, StanzaWrite},
    stanzafilter::StanzaFilter,
    websocket::outgoing::websocket_connect,
};

#[derive(Deserialize)]
struct Config {
    listen: SocketAddr,
    target: SocketAddr,
    server_name: String,
    origin: Option<String>,
    #[serde(deserialize_with = "deserialize_uri")]
    uri: Uri,
    #[serde(default = "default_max_stanza_size_bytes")]
    max_stanza_size_bytes: usize,
    log_level: Option<String>,
    log_style: Option<String>,
}

fn deserialize_uri<'de, D>(deserializer: D) -> Result<Uri, D::Error>
where
    D: Deserializer<'de>,
{
    let uri = String::deserialize(deserializer)?;
    uri.parse::<Uri>().map_err(serde::de::Error::custom)
}

fn default_max_stanza_size_bytes() -> usize {
    262_144
}

impl Config {
    fn parse<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut f = File::open(path)?;
        let mut input = String::new();
        f.read_to_string(&mut input)?;
        Ok(toml::from_str(&input)?)
    }
}

struct RuntimeCfg {
    config: Config,
    origin: String,
    outgoing_verifier_config: OutgoingVerifierConfig,
}

async fn handle_outgoing_connection(
    stream: tokio::net::TcpStream,
    client_addr: &mut Context<'_>,
    config: &RuntimeCfg,
) -> Result<()> {
    info!("{} connected", client_addr.log_from());
    let in_filter = StanzaFilter::new(config.config.max_stanza_size_bytes);

    let (in_rd, in_wr) = {
        let (in_rd, in_wr) = tokio::io::split(stream);
        (StanzaRead::new(in_rd), StanzaWrite::new(in_wr))
    };

    let (out_wr, out_rd) = websocket_connect(
        config.config.target,
        &config.config.server_name,
        &config.config.uri,
        &config.origin,
        &config.outgoing_verifier_config,
    )
    .await?;

    // proxy to direct TLS instead
    // socat TCP4-LISTEN:5222,reuseaddr,fork OPENSSL:10.16.19.1:5223,verify=0,commonname=burtrum.org
    /*
    let (out_wr, out_rd) = xmpp_proxy::tls::outgoing::tls_connect(
        config.config.target,
        &config.config.server_name,
        &config.outgoing_verifier_config,
    )
    .await?;
    */

    shuffle_rd_wr_filter_only(
        in_rd,
        in_wr,
        out_rd,
        out_wr,
        true,
        config.config.max_stanza_size_bytes,
        client_addr,
        in_filter,
    )
    .await
}

#[tokio::main]
//#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let cfg_path = std::env::args_os().nth(1);
    let cfg_path = cfg_path.unwrap_or_else(|| OsString::from("xmpp-bench-proxy.toml"));
    let config = Config::parse(&cfg_path).expect("invalid config file");

    let env = Env::default()
        .filter_or("XMPP_PROXY_LOG_LEVEL", "error")
        .write_style_or("XMPP_PROXY_LOG_STYLE", "never");
    let mut builder = Builder::from_env(env);
    builder.target(Target::Stdout);
    if let Some(ref log_level) = config.log_level {
        builder.parse_filters(log_level);
    }
    if let Some(ref log_style) = config.log_style {
        builder.parse_write_style(log_style);
    }
    // todo: config for this: builder.format_timestamp(None);
    builder.init();

    let outgoing_config = OutgoingConfig {
        max_stanza_size_bytes: config.max_stanza_size_bytes,
        certs_key: Arc::new(CertsKey {}),
    };

    let listener = TcpListener::bind(config.listen).await?;

    let runtime_cfg: &'static RuntimeCfg = Box::leak(Box::new(RuntimeCfg {
        origin: config
            .origin
            .as_ref()
            .unwrap_or(&config.server_name)
            .to_string(),
        config,
        outgoing_verifier_config: outgoing_config
            .with_custom_certificate_verifier(true, Arc::new(UnsafeNoopServerCertVerifier)),
    }));

    loop {
        let (stream, client_addr) = listener.accept().await?;
        tokio::spawn(async move {
            let mut client_addr = Context::new("unk-out", client_addr);
            if let Err(e) = handle_outgoing_connection(stream, &mut client_addr, runtime_cfg).await
            {
                error!("{} {}", client_addr.log_from(), e);
            }
        });
    }
}

struct UnsafeNoopServerCertVerifier;

impl rustls::client::ServerCertVerifier for UnsafeNoopServerCertVerifier {
    fn verify_server_cert(
        &self,
        _: &Certificate,
        _: &[Certificate],
        _: &ServerName,
        _: &mut dyn Iterator<Item = &[u8]>,
        _: &[u8],
        _: SystemTime,
    ) -> std::result::Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}
