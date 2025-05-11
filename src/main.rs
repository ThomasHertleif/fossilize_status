use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    process::Command,
};
use tracing::{Instrument, info, instrument};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

#[derive(Debug, Serialize, Deserialize)]
struct AppList {
    applist: Apps,
}

impl AppList {
    fn find_app(&self, app: u64) -> Option<&App> {
        self.applist.apps.iter().find(|a| a.appid == app)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Apps {
    apps: Vec<App>,
}

#[derive(Debug, Serialize, Deserialize)]
struct App {
    appid: u64,
    name: String,
}

fn get_cache_dir() -> PathBuf {
    let cache_base_dir = env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or("~".to_string());
            PathBuf::from(home).join(".cache")
        });

    cache_base_dir.join("fossilize_status")
}

fn get_cache_filename() -> PathBuf {
    get_cache_dir().join("steam_applist.json")
}

fn get_steam_app_id() -> Option<String> {
    // return Some("220".to_string());
    let output = Command::new("ps").args(["-ef"]).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.to_lowercase().contains("fossilize_replay")
            && line.contains("steamapps/shadercache/")
        {
            let parts: Vec<&str> = line.split("steamapps/shadercache/").collect();
            if parts.len() > 1 {
                let path_parts: Vec<&str> = parts[1].split('/').collect();
                if !path_parts.is_empty() {
                    return Some(path_parts[0].to_string());
                }
            }
        }
    }

    None
}

#[instrument()]
fn cache_applist() -> Result<AppList> {
    let res: AppList =
        reqwest::blocking::get("https://api.steampowered.com/ISteamApps/GetAppList/v2/")
            .context("fetching")?
            .json()
            .context("parsing")?;

    fs::create_dir_all(get_cache_dir()).context("create cache dir")?;

    let file = BufWriter::new(File::create(get_cache_filename())?);
    serde_json::to_writer(file, &res).context("writing json file")?;

    Ok(res)
}

#[instrument()]
fn get_cached() -> Result<AppList> {
    #[instrument()]
    fn read() -> Result<String> {
        fs::read_to_string(get_cache_filename()).context("failed to read file")
    }

    #[instrument(skip_all)]
    fn parse(text: String) -> Result<AppList> {
        serde_json::from_str::<AppList>(&text).context("failed to parse file")
    }

    read().and_then(parse)
}

fn get_or_cache() -> Result<AppList> {
    get_cached().or_else(|error| {
        info!(?error, "could not read cached thingy gonna cache");
        cache_applist()
    })
}

fn get_game_name(app_id: u64) -> Result<Option<String>> {
    // Search for the app in the app list
    let data = get_or_cache().context("get data")?;
    if let Some(app) = data.find_app(app_id) {
        return Ok(Some(app.name.clone()));
    }

    // If app not found in cache, try updating cache
    let data = cache_applist().context("refresh cache")?;
    if let Some(app) = data.applist.apps.iter().find(|app| app.appid == app_id) {
        return Ok(Some(app.name.clone()));
    }

    Ok(None)
}

fn main() -> Result<()> {
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(std::io::stderr),
        )
        .init();

    match get_steam_app_id() {
        Some(app_id) => {
            info!("Found Steam App ID: {}", app_id);
            let app_id: u64 = app_id.parse().expect("invalid id");

            match get_game_name(app_id)? {
                Some(game_name) => info!("Game Name: {}", game_name),
                None => info!("Could not find game name"),
            }
        }
        None => info!("No fossilize process found"),
    }

    Ok(())
}
