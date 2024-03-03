use std::string::String;
use std::collections::HashMap;
use std::process::Child;
use egui::{Color32, CursorIcon, Style, Visuals};
use linked_hash_map::LinkedHashMap;
use crate::counter_strike::{create_server_process, CS2APPID};
use crate::{counter_strike, steam, utils};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CS2ServerPrestarterApp {
    app_id: u32,

    // 1/0
    insecure: bool,

    // 1/0
    mp_autokick: bool,

    //     0 - default, only in buy zones
    //     1 - buy anywhere for all teams
    //     2 - buy anywhere for terrorists only
    //     3 - buy anywhere for counter-terrorists only
    mp_buy_anywhere: u8,

    // seconds
    mp_buytime: u32,

    // seconds
    mp_c4timer: u32,

    // seconds
    mp_freezetime: u32,

    // 1/0
    mp_friendlyfire: bool,

    // int
    mp_maxrounds: u32,

    //     0 - no random spawns
    //     1 - everyone spawns in random locations (like deathmatch)
    //     2 - only terrorists spawn at random locations, CTs spawn at their spawn
    //     3 - only CTs spawn at random locations, terrorists spawn at their spawn
    mp_randomspawn: u8,

    // seconds
    mp_roundtime: u32,

    // seconds
    mp_warmuptime: u32,

    mp_endwarmup_player_count: u32,

    // number, tickrate
    sv_minupdaterate: u32,

    // "0" to disable or password
    sv_password: String,

    game_alias: String,

    map_name: String,
    game_path_str: String,

    #[serde(skip)]
    game_path: String,

    #[serde(skip)]
    error_title: String,

    #[serde(skip)]
    error_msg: String,

    #[serde(skip)]
    ready: bool,

    #[serde(skip)]
    is_server_running: bool,

    #[serde(skip)]
    available_maps: Vec<String>,

    #[serde(skip)]
    error_popup_open: bool,

    #[serde(skip)]
    available_steam_apps: LinkedHashMap<u32, steamlocate::App>,
}


impl Default for CS2ServerPrestarterApp {
    fn default() -> Self {
        let mut game_path: String = String::new();
        let mut available_maps: Vec<String> = Vec::new();
        let mut ready = false;
        let mut available_apps: LinkedHashMap<u32, steamlocate::App> = LinkedHashMap::new();

        let (state, error, error_title) = match steam::get_steam_dir_for_app(CS2APPID) {
            Ok(res) => {
                game_path = res;
                (steam::ReadyState::Ready, "".to_string(), "".to_string())
            }
            Err(err) => {
                match err {
                    steamlocate::Error::FailedLocate(_) | steamlocate::Error::InvalidSteamDir(_) => {
                        (
                            steam::ReadyState::NoSteam,
                            "Unable to find SteamLibrary directory. Are you sure you have Steam installed?".to_string(),
                            "Steam not found".to_string()
                        )
                    }
                    steamlocate::Error::MissingExpectedApp { app_id } => {
                        (
                            steam::ReadyState::NoGame,
                            format!("Unable to find app with id {app_id}. \nAre you sure it is installed?"),
                            "Game not found".to_string()
                        )
                    }
                    _ => {
                        (
                            steam::ReadyState::NoSteam,
                            format!("Unknown error occurred: {err}"),
                            "Unknown error".to_string()
                        )
                    }
                }
            }
        };

        match state {
            steam::ReadyState::Ready => {
                available_maps = match counter_strike::get_available_maps(&game_path) {
                    None => {
                        Vec::new()
                    }
                    Some(res) => { res }
                };
                ready = true;
            }
            steam::ReadyState::NoGame => {
                available_apps = steam::get_available_apps().unwrap();
            }
            steam::ReadyState::NoSteam => {}
        };

        Self {
            game_path,
            ready,
            app_id: 730,
            game_path_str: "".to_string(),
            map_name: "de_dust2".to_string(),

            insecure: true,
            mp_autokick: true,
            mp_buy_anywhere: 0u8,
            mp_buytime: 15,
            mp_c4timer: 40,
            mp_freezetime: 20,
            mp_friendlyfire: true,
            mp_maxrounds: 32,
            mp_endwarmup_player_count: 2,
            mp_randomspawn: 0,
            mp_roundtime: 115,
            mp_warmuptime: 15,
            sv_minupdaterate: 64,
            sv_password: "0".to_string(),

            game_alias: "competitive".to_string(),

            error_msg: error,
            error_title,
            available_maps,
            is_server_running: false,

            error_popup_open: true,
            available_steam_apps: available_apps,
        }
    }
}

impl CS2ServerPrestarterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let style = Style {
            visuals: Visuals::dark(),
            ..Style::default()
        };
        cc.egui_ctx.set_style(style);
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }
}

impl eframe::App for CS2ServerPrestarterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.ready {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Window::new(&self.error_title)
                        .vscroll(true)
                        .collapsible(false)
                        .resizable(false)
                        .movable(false)
                        .show(ctx, |ui| {
                            egui::ScrollArea::vertical().max_height(480.0).auto_shrink(true).show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.error_msg.as_str())
                                        .desired_width(640f32)
                                );
                                egui::Grid::new("")
                                    .num_columns(2)
                                    .show(ui, |ui| {
                                        for k in self.available_steam_apps.keys() {
                                            ui.label(k.to_string());
                                            ui.label(self.available_steam_apps[k].name.clone().unwrap());
                                            if ui.button("Select").on_hover_cursor(CursorIcon::PointingHand).clicked() {
                                                self.app_id = *k;
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                        });
                });
            }

            ui.heading("CS2 Server Settings");

            ui.add_enabled_ui(self.ready, |ui| {
                ui.horizontal(|ui| {
                    let name_label = ui.label("Game path: ");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.game_path)
                            .desired_width(400.0)
                            .text_color(Color32::BLACK)
                    ).labelled_by(name_label.id);
                    if ui.button("Open")
                        .on_hover_text("Open game directory")
                        .clicked() {}
                });

                egui::Grid::new("")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("insecure")
                            .on_hover_text("Disable VAC on your server. If enabled you have to add `-insecure` to your CS2 start params")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.checkbox(&mut self.insecure, "");
                        ui.end_row();

                        ui.label("mp_autokick")
                            .on_hover_text("Kick for AFK or team dmg")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.checkbox(&mut self.mp_autokick, "");
                        ui.end_row();

                        ui.label("mp_buy_anywhere")
                            .on_hover_text("Can anyone buy anywhere on map.")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.mp_buy_anywhere, 0, "Only buy zone");
                            ui.radio_value(&mut self.mp_buy_anywhere, 1, "All teams");
                            ui.radio_value(&mut self.mp_buy_anywhere, 2, "T only");
                            ui.radio_value(&mut self.mp_buy_anywhere, 3, "CT only");
                        });
                        ui.end_row();

                        ui.label("mp_buytime")
                            .on_hover_text("Time to buy after freezetime in seconds")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.add(
                            egui::DragValue::new(&mut self.mp_buytime)
                                .speed(0.1)
                                .clamp_range(0..=self.mp_roundtime)
                        );
                        ui.end_row();

                        ui.label("mp_c4timer")
                            .on_hover_text("Seconds before explosion after planted")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.add(
                            egui::DragValue::new(&mut self.mp_c4timer)
                                .speed(0.1)
                                .clamp_range(0..=self.mp_roundtime)
                        );
                        ui.end_row();

                        ui.label("mp_freezetime")
                            .on_hover_text("Freeze time before round start in seconds")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.add(
                            egui::DragValue::new(&mut self.mp_freezetime)
                                .speed(0.1)
                                .clamp_range(0..=self.mp_roundtime)
                        );
                        ui.end_row();

                        ui.label("mp_friendlyfire")
                            .on_hover_text("Enable friendly fire or not")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.checkbox(&mut self.mp_friendlyfire, "");
                        ui.end_row();

                        ui.label("mp_maxrounds")
                            .on_hover_text("Max rounds (for both teams summary). Team switch at half of this value")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.add(
                            egui::DragValue::new(&mut self.mp_maxrounds)
                                .speed(0.1)
                                .clamp_range(0..=4096)
                        );
                        ui.end_row();

                        ui.label("mp_randomspawn")
                            .on_hover_text("Enable random spawn")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.mp_randomspawn, 0, "Disabled")
                                .on_hover_text("no random spawns");
                            ui.radio_value(&mut self.mp_randomspawn, 1, "Everyone")
                                .on_hover_text("everyone spawns in random locations (like deathmatch)");
                            ui.radio_value(&mut self.mp_randomspawn, 2, "T only")
                                .on_hover_text("only terrorists spawn at random locations, CTs spawn at their spawn");

                            ui.radio_value(&mut self.mp_randomspawn, 3, "CT only")
                                .on_hover_text("only CTs spawn at random locations, terrorists spawn at their spawn");
                        });
                        ui.end_row();

                        ui.label("mp_roundtime")
                            .on_hover_text("Round time in seconds")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.add(
                            egui::DragValue::new(&mut self.mp_roundtime)
                                .speed(0.1)
                                .clamp_range(0..=3600)
                        );
                        ui.end_row();

                        ui.label("mp_warmuptime")
                            .on_hover_text("Warmup duration in seconds")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.add(
                            egui::DragValue::new(&mut self.mp_warmuptime)
                                .speed(0.1)
                                .clamp_range(0..=3600)
                        );
                        ui.end_row();
                        ui.label("mp_endwarmup_player_count")
                            .on_hover_text("Players to connect to skip warmup")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.add(
                            egui::DragValue::new(&mut self.mp_endwarmup_player_count)
                                .speed(0.1)
                                .clamp_range(0..=3600)
                        );
                        ui.end_row();

                        ui.label("sv_minupdaterate")
                            .on_hover_text("Minimal update rate for server")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.add(
                            egui::DragValue::new(&mut self.sv_minupdaterate)
                                .speed(0.1)
                                .clamp_range(0..=4096)
                        );
                        ui.end_row();

                        ui.label("sv_password")
                            .on_hover_text("Password to join server, type \"0\" for no password")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.text_edit_singleline(&mut self.sv_password);
                        ui.end_row();

                        ui.label("map")
                            .on_hover_text("Map to play on")
                            .on_hover_cursor(CursorIcon::Default);
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", self.map_name))
                            .show_ui(ui, |ui| {
                                for one_map in &self.available_maps {
                                    ui.selectable_value(&mut self.map_name, one_map.to_string(), one_map);
                                }
                            });
                        if ui.button("Open")
                            .on_hover_text("Open maps directory")
                            .clicked() {}
                        ui.end_row();

                        ui.label("game_alias")
                            .on_hover_text("Game type")
                            .on_hover_cursor(CursorIcon::Default);
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.game_alias, String::from("competitive"), "Competitive");
                            ui.radio_value(&mut self.game_alias, String::from("wingman"), "Wingman");
                            ui.radio_value(&mut self.game_alias, String::from("casual"), "Casual");
                            ui.radio_value(&mut self.game_alias, String::from("custom"), "Custom");
                        });
                        ui.end_row();
                    });

                let start_server = ui.button("Run server")
                    .on_disabled_hover_text("No CS2 found!")
                    .on_hover_text("Start CS2 server with selected map and settings");

                let mut server_process: Option<Child> = None;

                if start_server.clicked() {
                    let mut args: Vec<String> = Vec::new();
                    args.push(String::from("-dedicated"));
                    args.push(String::from("-insecure"));

                    args.extend_from_slice(&[
                        String::from("+map"),
                        String::from(&self.map_name)
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_autokick"),
                        String::from(&utils::bool_to_str(self.mp_autokick))
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_buy_anywhere"),
                        String::from(&self.mp_buy_anywhere.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_buytime"),
                        String::from(&self.mp_buytime.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_c4timer"),
                        String::from(&self.mp_c4timer.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_freezetime"),
                        String::from(&self.mp_freezetime.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_friendlyfire"),
                        String::from(&utils::bool_to_str(self.mp_friendlyfire))
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_maxrounds"),
                        String::from(&self.mp_maxrounds.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_randomspawn"),
                        String::from(&self.mp_randomspawn.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_roundtime"),
                        String::from(&utils::seconds_to_minutes_str(self.mp_roundtime))
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_warmuptime"),
                        String::from(&self.mp_warmuptime.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+mp_endwarmup_player_count"),
                        String::from(&self.mp_endwarmup_player_count.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+sv_minupdaterate"),
                        String::from(&self.sv_minupdaterate.to_string())
                    ]);
                    args.extend_from_slice(&[
                        String::from("+sv_password"),
                        String::from(&self.sv_password.to_string())
                    ]);

                    let mut envs: HashMap<String, String> = HashMap::new();
                    envs.insert(String::from("game_alias"), String::from(&self.game_alias));

                    match create_server_process(&self.game_path, args, envs) {
                        Ok(res) => {
                            self.is_server_running = true;
                            server_process = Some(res);
                        }
                        Err(_) => {}
                    }
                };

                ui.add_enabled_ui(self.is_server_running, |ui| {
                    let _ = ui.hyperlink(
                        format!(
                            "steam://connect/{url}:{port}/{password}",
                            url = "127.0.0.1",
                            port = 27015,
                            password = self.sv_password
                        )
                    );
                });
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
