#[cfg(target_family = "wasm")]
use crate::{
    launch_options::{ClientLaunchOptions, SharedLaunchOptions},
    launch_options::{SerializableClientLaunchOptions, SerializableSharedLaunchOptions},
};
use bevy::prelude::{App, Commands, Plugin, Res, Val, default, info};
use client::app::build_client_app;
use lightyear::{
    client::config::{ClientConfig, NetcodeConfig as ClientNetcodeConfig},
    connection::client::NetConfig as ClientNetConfig,
    prelude::{
        LinkConditionerConfig, SharedConfig, TickConfig,
        client::{
            Authentication, ClientTransport, InterpolationConfig, IoConfig as ClientIoConfig,
            PredictionConfig,
        },
    },
};
use ron::de::from_str;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use wasm_bindgen::prelude::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, console};

const CLIENT_CONFIG_PATH: &str = "./options/web_client_options.ron";
const SHARED_CONFIG_PATH: &str = "./options/shared_options.ron";

fn extract_client_id() -> Option<u64> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    let client_id = params.get("client_id")?;
    client_id.parse::<u64>().ok()
}

async fn fetch_config(path: &str) -> Result<String, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(path, &opts)?;
    request.headers().set("Accept", "application/ron")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;

    if !resp.ok() {
        console::log_1(&format!("Failed to fetch config from {}: {}", path, resp.status()).into());
        return Err(JsValue::from_str(&format!(
            "HTTP error! status: {}",
            resp.status()
        )));
    }

    let text = JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().unwrap())
}

async fn load_client_config() -> Result<ClientLaunchOptions, JsValue> {
    match fetch_config(CLIENT_CONFIG_PATH).await {
        Ok(text) => {
            let serializable_config: SerializableClientLaunchOptions = match from_str(&text) {
                Ok(config) => config,
                Err(e) => {
                    console::log_1(&format!("Error parsing client config: {}", e).into());
                    return Ok(ClientLaunchOptions::default());
                }
            };
            Ok(ClientLaunchOptions::from(serializable_config))
        }
        Err(e) => {
            console::log_1(&format!("Using default client config: {:?}", e).into());
            Ok(ClientLaunchOptions::default())
        }
    }
}

async fn load_shared_config() -> Result<SharedLaunchOptions, JsValue> {
    match fetch_config(SHARED_CONFIG_PATH).await {
        Ok(text) => {
            let serializable_config: SerializableSharedLaunchOptions = match from_str(&text) {
                Ok(config) => config,
                Err(e) => {
                    console::log_1(&format!("Error parsing shared config: {}", e).into());
                    return Ok(SharedLaunchOptions::default());
                }
            };
            Ok(SharedLaunchOptions::from(serializable_config))
        }
        Err(e) => {
            console::log_1(&format!("Using default shared config: {:?}", e).into());
            Ok(SharedLaunchOptions::default())
        }
    }
}

pub fn run() {
    console_error_panic_hook::set_once();
    console::log_1(&"WASM initializing...".into());

    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = initialize_game().await {
            console::log_1(&format!("Failed to initialize game: {:?}", e).into());
        }
    });
}

async fn initialize_game() -> Result<(), JsValue> {
    let client_launch_options = load_client_config().await?;
    let shared_launch_options = load_shared_config().await?;

    let certificate_digest = match &client_launch_options.certificate_digest {
        Some(digest) => {
            console::log_2(
                &"Using certificate digest".into(),
                &JsValue::from_str(&digest),
            );
            digest.clone()
        }
        None => {
            console::log_1(&"No certificate digest found in options.".into());
            return Err(JsValue::from_str("Missing certificate digest"));
        }
    };

    let client_id = extract_client_id().unwrap_or(293857);
    console::log_2(
        &"Using client ID".into(),
        &JsValue::from_f64(client_id as f64),
    );

    let shared_config = SharedConfig {
        server_replication_send_interval: shared_launch_options.server_replication_send_interval,
        client_replication_send_interval: shared_launch_options.client_replication_send_interval,
        tick: TickConfig {
            tick_duration: shared_launch_options.simulation_update_frequency,
        },
    };

    let transport_config = ClientIoConfig::from_transport(ClientTransport::WebTransportClient {
        client_addr: SocketAddr::new(
            IpAddr::V4(client_launch_options.listen_addr),
            client_launch_options.listen_port,
        ),
        server_addr: SocketAddr::new(
            IpAddr::V4(client_launch_options.server_addr),
            client_launch_options.server_port,
        ),
        certificate_digest: certificate_digest.to_owned(),
    });

    let auth = Authentication::Manual {
        server_addr: SocketAddr::new(
            IpAddr::V4(client_launch_options.server_addr),
            client_launch_options.server_port,
        ),
        client_id,
        private_key: shared_launch_options.key,
        protocol_id: shared_launch_options.protocol_id,
    };

    let client_config = ClientConfig {
        shared: shared_config,
        net: ClientNetConfig::Netcode {
            auth,
            config: ClientNetcodeConfig {
                token_expire_secs: -1,
                client_timeout_secs: 5,
                ..default()
            },
            io: transport_config,
        },
        prediction: PredictionConfig::default()
            .with_correction_ticks_factor(client_launch_options.correction_ticks_factor),
        interpolation: InterpolationConfig {
            min_delay: client_launch_options.min_delay,
            send_interval_ratio: 0.,
        },
        ..default()
    };

    console::log_1(&"Starting client app...".into());
    build_client_app(client_config, client_launch_options.asset_path).run();

    Ok(())
}
