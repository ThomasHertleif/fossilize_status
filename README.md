# fossilize_status
Python script that checks which steam game is currently processing shaders

## Usage

Run `python fossilize_status.py`
If a game is processing shaders, it will be displayed in the console.

## Installation

You can chmod +x fossilize_status.py and place it in your PATH.
For example `~/.local/bin`

## How does it work?

The script uses `ps` to get the current appId `fossilize_replay` process.

Then gets all appIds from the [Steam API](https://partner.steamgames.com/doc/webapi/ISteamApps#GetAppList) and caches them locally in `XDG_CACHE_HOME/fossilize_status/` or `~/.cache/fossilize_status/`.

It will lookup the appId from the json.
If it is not found, the cache will be updated.

The cache file is ~14MB.
