#!/usr/bin/env python

import subprocess
import requests

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

def get_game_name(app_id):
    try:
        url = f"https://store.steampowered.com/api/appdetails?appids={app_id}"
        response = requests.get(url)
        if response.status_code == 200:
            data = response.json()
            if data[app_id]['success']:
                return data[app_id]['data']['name']
    except:
        pass
    return None

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
