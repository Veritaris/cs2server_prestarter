use linked_hash_map::LinkedHashMap;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ReadyState {
    NoSteam,
    NoGame,
    Ready,
}

#[derive(Debug)]
pub enum ServerCreationError {
    NoExecutableFound,
    UnsupportedOS,
}

pub fn get_available_apps() -> Result<LinkedHashMap<u32, steamlocate::App>, steamlocate::Error> {
    let steam_dir = steamlocate::SteamDir::locate()?;
    let mut available_apps: LinkedHashMap<u32, steamlocate::App> = LinkedHashMap::new();

    for library in steam_dir.libraries()? {
        let library = library?;

        for app in library.apps() {
            let mut app = app?;
            app.install_dir = String::from(library.resolve_app_dir(&app).to_str().unwrap());
            available_apps.insert(app.app_id, app.clone());
        }
    };

    let mut sorted: Vec<(u32, steamlocate::App)> = available_apps.iter()
        .map(|t| (*t.0, t.1.clone()))
        .collect();

    sorted.sort_by_key(|e| e.0);
    available_apps = LinkedHashMap::from_iter(sorted);

    return Ok(available_apps);
}


pub fn get_steam_apps_id_with_paths() -> Result<LinkedHashMap<u32, String>, steamlocate::Error> {
    return Ok(
        LinkedHashMap::from_iter(
            get_available_apps()?.iter()
                .map(|e| (*e.0, e.1.clone().install_dir))
                .collect::<Vec<(u32, String)>>()
        )
    );
}


pub fn get_steam_dir_for_app(target_app_id: &u32) -> Result<String, steamlocate::Error> {
    return match get_steam_apps_id_with_paths()?.get(&target_app_id) {
        None => {
            Err(steamlocate::Error::MissingExpectedApp { app_id: *target_app_id })
        }
        Some(res) => {
            Ok(String::from(res))
        }
    };
}
