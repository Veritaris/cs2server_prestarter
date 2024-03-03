use std::collections::HashMap;
use std::fs;
use std::path::{Path};
use std::process::{Child, Command};
use crate::steam;


pub const CS2APPID: &u32 = &730;
pub const MAP_EXT: &str = &".vpk";

pub const MAPS_DIR_PATH: &str = &"game/csgo/maps";
const BINARY_DIR_PATH: &str = &"game/bin/win64";
const BINARY_NAME: &str = &"cs2.exe";

pub fn get_available_maps(game_path: &String) -> Option<Vec<String>> {
    let maps_dir = Path::new(game_path).join(MAPS_DIR_PATH);

    return match fs::read_dir(maps_dir) {
        Ok(files) => {
            Some(
                files.into_iter()
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap().file_name().into_string().unwrap())
                    .filter(|r| r.ends_with(MAP_EXT))
                    .map(|r| { r.replace(MAP_EXT, "") })
                    .collect()
            )
        }
        Err(_) => { None }
    };
}

pub fn create_server_process(game_path: &String, args: Vec<String>, envs: HashMap<String, String>) -> Result<Child, steam::ServerCreationError> {
    let game_exe_path = Path::new(game_path).join(BINARY_DIR_PATH).join(BINARY_NAME);

    if !game_exe_path.exists() {
        return Err(steam::ServerCreationError::NoExecutableFound);
    }

    if !cfg!(target_os = "windows") {
        return Err(steam::ServerCreationError::UnsupportedOS);
    }

    let process = Command::new(game_exe_path.canonicalize().unwrap())
        .envs(envs)
        .args(args)
        .spawn()
        .expect("something went wrongly...");
    return Ok(process);
}