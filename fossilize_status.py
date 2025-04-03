#!/usr/bin/env python

import subprocess
import requests
import os
import json

cache_base_dir = os.getenv('XDG_CACHE_HOME', os.path.expanduser('~/.cache'))
cache_dir = os.path.join(cache_base_dir, 'fossilize_status')
cache_file = os.path.join(cache_dir, 'steam_applist.json')

def get_steam_app_id():
    try:
        # Get process list using ps command
        ps = subprocess.Popen(['ps', '-ef'], stdout=subprocess.PIPE).communicate()[0]
        processes = ps.decode('utf-8').split('\n')

        # Search for fossilize process
        for proc in processes:
            if 'fossilize_replay' in proc.lower():
                # Look for Steam App ID in process path
                if 'steamapps/shadercache/' in proc:
                    path_parts = proc.split('steamapps/shadercache/')[1].split('/')
                    if len(path_parts) > 0:
                        return path_parts[0]
    except (subprocess.SubprocessError, FileNotFoundError, IndexError):
        pass
    return None

def cache_applist():
    # fetch new data
    try:
        response = requests.get('https://api.steampowered.com/ISteamApps/GetAppList/v2/')
        data = response.json()

        # Ensure cache directory exists
        os.makedirs(cache_dir, exist_ok=True)

        # Save to cache
        with open(cache_file, 'w') as f:
            json.dump(data, f)

        return data
    except:
        print("Failed to fetch Steam app list")
        return None

def get_game_name(app_id):
    try:
        # Try to load from cache first
        with open(cache_file, 'r') as f:
            data = json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        # If cache doesn't exist or is invalid, fetch new data
        data = cache_applist()
        if not data:
            return None

    # Search for the app in the app list
    for app in data['applist']['apps']:
        if str(app['appid']) == str(app_id):
            return app['name']

    # If app not found in cache, try updating cache
    data = cache_applist()
    if data:
        for app in data['applist']['apps']:
            if str(app['appid']) == str(app_id):
                return app['name']

    return None

def main():
    steam_app_id = get_steam_app_id()
    if steam_app_id:
        print(f"Found Steam App ID: {steam_app_id}")
    else:
        print("No fossilize process found")

    if steam_app_id:
        game_name = get_game_name(steam_app_id)
        if game_name:
            print(f"Game Name: {game_name}")
        else:
            print("Could not find game name")

if __name__ == "__main__":
    main()
