use log::Log;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct BlogApp {
    auth_tab: AuthButtons,
    username: String,
    email: String,
    password: String,
    token: Option<String>,

    login_status: LoginStatus,
}

impl Default for BlogApp {
    fn default() -> Self {
        Self {
            auth_tab: AuthButtons::Login,
            username: String::new(),
            email: String::new(),
            password: String::new(),
            token: None,
            login_status: LoginStatus::LoggedOut,
        }
    }
}

impl BlogApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AuthButtons {
    Login,
    Register,
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
pub enum LoginStatus {
    Logined,
    WrongPassword,
    LoggedOut,
}

pub fn auth_buttons(current_value: &mut AuthButtons, status: &LoginStatus, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.selectable_value(current_value, AuthButtons::Login, "Login")
            .on_hover_text("Use the dark mode theme");

        ui.selectable_value(current_value, AuthButtons::Register, "Register")
            .on_hover_text("Use the light mode theme");
        let text = match status {
            LoginStatus::Logined => "logined",
            LoginStatus::LoggedOut => "logged out",
            LoginStatus::WrongPassword => "wrong password",
        };
        ui.label(text)
    });
}

impl eframe::App for BlogApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Blog");
            ui.add_space(8.0);

            auth_buttons(&mut self.auth_tab, &self.login_status, ui);
            ui.add_space(8.0);

            match self.auth_tab {
                AuthButtons::Login => {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.username);
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                    ui.add_space(8.0);
                    if ui.button("Login").clicked() {
                        // TODO: call login API
                    }
                }
                AuthButtons::Register => {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.username);
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.email);
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                    ui.add_space(8.0);
                    if ui.button("Register").clicked() {
                        // TODO: call register API
                    }
                }
            }
        });
    }
}
