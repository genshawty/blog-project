use std::sync::mpsc;

use crate::api::{ApiClient, AuthResponse, PostListResponse, PostResponse};

// here comments about what each paramemer is doing
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BlogApp {
    #[serde(skip)]
    auth_tab: AuthTab,
    username: String,
    #[serde(skip)]
    email: String,
    #[serde(skip)]
    password: String,
    token: Option<String>,
    logged_in_user: Option<String>,
    logged_in_user_id: Option<String>,
    #[serde(skip)]
    status_message: String,

    #[serde(skip)]
    api: ApiClient,
    #[serde(skip)]
    auth_rx: Option<mpsc::Receiver<Result<AuthResponse, String>>>,
    #[serde(skip)]
    posts: Vec<PostResponse>,
    #[serde(skip)]
    posts_rx: Option<mpsc::Receiver<Result<PostListResponse, String>>>,
    #[serde(skip)]
    posts_loaded: bool,
    #[serde(skip)]
    new_post_title: String,
    #[serde(skip)]
    new_post_content: String,
    #[serde(skip)]
    create_post_rx: Option<mpsc::Receiver<Result<PostResponse, String>>>,
    #[serde(skip)]
    delete_rx: Option<mpsc::Receiver<Result<(), String>>>,
    #[serde(skip)]
    update_rx: Option<mpsc::Receiver<Result<PostResponse, String>>>,
    #[serde(skip)]
    editing_post_id: Option<String>,
    #[serde(skip)]
    edit_title: String,
    #[serde(skip)]
    edit_content: String,
}

impl Default for BlogApp {
    fn default() -> Self {
        Self {
            auth_tab: AuthTab::Login,
            username: String::new(),
            email: String::new(),
            password: String::new(),
            token: None,
            logged_in_user: None,
            logged_in_user_id: None,
            status_message: String::new(),
            api: ApiClient::new(),
            auth_rx: None,
            posts: Vec::new(),
            posts_rx: None,
            posts_loaded: false,
            new_post_title: String::new(),
            new_post_content: String::new(),
            create_post_rx: None,
            delete_rx: None,
            update_rx: None,
            editing_post_id: None,
            edit_title: String::new(),
            edit_content: String::new(),
        }
    }
}

impl BlogApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn spawn_login(&mut self, ctx: &egui::Context) {
        let (tx, rx) = mpsc::channel();
        self.auth_rx = Some(rx);

        let api = self.api.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let ctx = ctx.clone();

        spawn_future(async move {
            let result = api.login(&username, &password).await;
            let _ = tx.send(result);
            ctx.request_repaint();
        });

        self.status_message = "Logging in...".to_string();
    }

    // here how this works shortly and why required
    fn spawn_register(&mut self, ctx: &egui::Context) {
        let (tx, rx) = mpsc::channel();
        self.auth_rx = Some(rx);

        let api = self.api.clone();
        let username = self.username.clone();
        let email = self.email.clone();
        let password = self.password.clone();
        let ctx = ctx.clone();

        spawn_future(async move {
            let result = api.register(&username, &email, &password).await;
            let _ = tx.send(result);
            ctx.request_repaint();
        });

        self.status_message = "Registering...".to_string();
    }

    // here how this works shortly and why required
    fn poll_auth_result(&mut self, ctx: &egui::Context) {
        let result = self.auth_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.auth_rx = None;
            match result {
                Ok(auth) => {
                    self.logged_in_user = Some(auth.user.username);
                    self.logged_in_user_id = Some(auth.user.id);
                    self.status_message = format!(
                        "Logged in as {}",
                        self.logged_in_user.as_deref().unwrap_or("?")
                    );
                    self.token = Some(auth.token);
                    self.password.clear();
                    self.spawn_load_posts(ctx);
                }
                Err(e) => {
                    self.status_message = e;
                }
            }
        }
    }

    fn poll_posts_result(&mut self) {
        let result = self.posts_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.posts_rx = None;
            match result {
                Ok(list) => {
                    self.posts = list.posts;
                    self.posts_loaded = true;
                }
                Err(e) => {
                    self.status_message = format!("Failed to load posts: {e}");
                }
            }
        }
    }

    fn spawn_load_posts(&mut self, ctx: &egui::Context) {
        let (tx, rx) = mpsc::channel();
        self.posts_rx = Some(rx);

        let api = self.api.clone();
        let ctx = ctx.clone();

        spawn_future(async move {
            let result = api.list_posts(100, 0).await;
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }

    fn spawn_create_post(&mut self, ctx: &egui::Context) {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };
        let (tx, rx) = mpsc::channel();
        self.create_post_rx = Some(rx);

        let api = self.api.clone();
        let title = self.new_post_title.clone();
        let content = self.new_post_content.clone();
        let ctx = ctx.clone();

        spawn_future(async move {
            let result = api.create_post(&token, &title, &content).await;
            let _ = tx.send(result);
            ctx.request_repaint();
        });

        self.status_message = "Creating post...".to_string();
    }

    fn poll_create_post_result(&mut self, ctx: &egui::Context) {
        let result = self
            .create_post_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.create_post_rx = None;
            match result {
                Ok(_) => {
                    self.new_post_title.clear();
                    self.new_post_content.clear();
                    self.status_message = "Post created!".to_string();
                    self.spawn_load_posts(ctx);
                }
                Err(e) => {
                    self.status_message = format!("Failed to create post: {e}");
                }
            }
        }
    }

    fn spawn_delete_post(&mut self, ctx: &egui::Context, post_id: &str) {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };
        let (tx, rx) = mpsc::channel();
        self.delete_rx = Some(rx);

        let api = self.api.clone();
        let post_id = post_id.to_string();
        let ctx = ctx.clone();

        spawn_future(async move {
            let result = api.delete_post(&token, &post_id).await;
            let _ = tx.send(result);
            ctx.request_repaint();
        });

        self.status_message = "Deleting post...".to_string();
    }

    fn poll_delete_result(&mut self, ctx: &egui::Context) {
        let result = self.delete_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.delete_rx = None;
            match result {
                Ok(()) => {
                    self.status_message = "Post deleted!".to_string();
                    self.spawn_load_posts(ctx);
                }
                Err(e) => {
                    self.status_message = format!("Failed to delete post: {e}");
                }
            }
        }
    }

    fn spawn_update_post(&mut self, ctx: &egui::Context) {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };
        let post_id = match &self.editing_post_id {
            Some(id) => id.clone(),
            None => return,
        };
        let (tx, rx) = mpsc::channel();
        self.update_rx = Some(rx);

        let api = self.api.clone();
        let title = self.edit_title.clone();
        let content = self.edit_content.clone();
        let ctx = ctx.clone();

        spawn_future(async move {
            let result = api.update_post(&token, &post_id, &title, &content).await;
            let _ = tx.send(result);
            ctx.request_repaint();
        });

        self.status_message = "Updating post...".to_string();
    }

    fn poll_update_result(&mut self, ctx: &egui::Context) {
        let result = self.update_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.update_rx = None;
            match result {
                Ok(_) => {
                    self.editing_post_id = None;
                    self.edit_title.clear();
                    self.edit_content.clear();
                    self.status_message = "Post updated!".to_string();
                    self.spawn_load_posts(ctx);
                }
                Err(e) => {
                    self.status_message = format!("Failed to update post: {e}");
                }
            }
        }
    }

    fn logout(&mut self) {
        self.token = None;
        self.logged_in_user = None;
        self.logged_in_user_id = None;
        self.posts.clear();
        self.posts_loaded = false;
        self.status_message = "Logged out".to_string();
    }
}

enum PostAction {
    Delete(String),
    StartEdit(String, String, String),
    Update,
    CancelEdit,
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AuthTab {
    Login,
    Register,
}

fn spawn_future(future: impl std::future::Future<Output = ()> + 'static) {
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(future);

    // #[cfg(not(target_arch = "wasm32"))]
    // std::thread::spawn(|| pollster::block_on(future));
}

impl eframe::App for BlogApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_auth_result(ctx);
        self.poll_posts_result();
        self.poll_create_post_result(ctx);
        self.poll_delete_result(ctx);
        self.poll_update_result(ctx);

        // Load posts on first frame if not yet loaded
        if !self.posts_loaded && self.posts_rx.is_none() {
            self.spawn_load_posts(ctx);
        }

        // Increase base font size
        let mut style = (*ctx.style()).clone();
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = 16 as f32;
        }
        ctx.set_style(style);

        let max_width = 700.0;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_max_width(max_width);

                ui.heading("Blog");
                ui.add_space(8.0);

                if let Some(user) = self.logged_in_user.clone() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Logged in as: {user}"));
                        if ui.button("Logout").clicked() {
                            self.logout();
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.auth_tab, AuthTab::Login, "Login");
                        ui.selectable_value(&mut self.auth_tab, AuthTab::Register, "Register");
                    });
                    ui.add_space(4.0);

                    let is_loading = self.auth_rx.is_some();

                    match self.auth_tab {
                        AuthTab::Login => {
                            ui.label("Username:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.username)
                                    .desired_width(max_width),
                            );
                            ui.label("Password:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.password)
                                    .password(true)
                                    .desired_width(max_width),
                            );
                            ui.add_space(8.0);
                            ui.add_enabled_ui(!is_loading, |ui| {
                                if ui.button("Login").clicked() {
                                    self.spawn_login(ctx);
                                }
                            });
                        }
                        AuthTab::Register => {
                            ui.label("Username:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.username)
                                    .desired_width(max_width),
                            );
                            ui.label("Email:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.email)
                                    .desired_width(max_width),
                            );
                            ui.label("Password:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.password)
                                    .password(true)
                                    .desired_width(max_width),
                            );
                            ui.add_space(8.0);
                            ui.add_enabled_ui(!is_loading, |ui| {
                                if ui.button("Register").clicked() {
                                    self.spawn_register(ctx);
                                }
                            });
                        }
                    }
                }

                if !self.status_message.is_empty() {
                    ui.add_space(8.0);
                    ui.label(&self.status_message);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Create Post form (only for logged-in users)
                if self.logged_in_user.is_some() {
                    ui.strong("New Post");
                    ui.add_space(4.0);
                    ui.label("Title:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_post_title)
                            .desired_width(max_width),
                    );
                    ui.label("Content:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.new_post_content)
                            .desired_width(max_width),
                    );
                    ui.add_space(4.0);
                    let is_creating = self.create_post_rx.is_some();
                    ui.add_enabled_ui(!is_creating, |ui| {
                        ui.horizontal(|ui| {
                            let can_create = !self.new_post_title.is_empty()
                                && !self.new_post_content.is_empty();
                            if ui
                                .add_enabled(can_create, egui::Button::new("Create"))
                                .clicked()
                            {
                                self.spawn_create_post(ctx);
                            }
                            if ui.button("Cancel").clicked() {
                                self.new_post_title.clear();
                                self.new_post_content.clear();
                            }
                        });
                    });
                    ui.add_space(8.0);
                }

                // Posts feed
                if self.posts_rx.is_some() && self.posts.is_empty() {
                    ui.label("Loading posts...");
                } else if self.posts.is_empty() {
                    ui.label("No posts yet.");
                } else {
                    let current_user_id = self.logged_in_user_id.clone();
                    let mut action: Option<PostAction> = None;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.set_max_width(max_width);
                        for post in &self.posts {
                            let is_owner = current_user_id
                                .as_ref()
                                .map_or(false, |uid| uid == &post.author_id);
                            let is_editing = self
                                .editing_post_id
                                .as_ref()
                                .map_or(false, |id| id == &post.id);

                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                if is_editing {
                                    ui.label("Title:");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.edit_title)
                                            .desired_width(f32::INFINITY),
                                    );
                                    ui.label("Content:");
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.edit_content)
                                            .desired_width(f32::INFINITY),
                                    );
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        let can_save = !self.edit_title.is_empty()
                                            && !self.edit_content.is_empty();
                                        if ui
                                            .add_enabled(can_save, egui::Button::new("Save"))
                                            .clicked()
                                        {
                                            action = Some(PostAction::Update);
                                        }
                                        if ui.button("Cancel").clicked() {
                                            action = Some(PostAction::CancelEdit);
                                        }
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        ui.strong(&post.title);
                                        if is_owner {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.small_button("Delete").clicked() {
                                                        action = Some(PostAction::Delete(
                                                            post.id.clone(),
                                                        ));
                                                    }
                                                    if ui.small_button("Edit").clicked() {
                                                        action = Some(PostAction::StartEdit(
                                                            post.id.clone(),
                                                            post.title.clone(),
                                                            post.content.clone(),
                                                        ));
                                                    }
                                                },
                                            );
                                        }
                                    });
                                    ui.add_space(4.0);
                                    ui.label(&post.content);
                                    ui.add_space(4.0);
                                    ui.weak(&post.created_at);
                                }
                            });
                            ui.add_space(4.0);
                        }
                    });

                    match action {
                        Some(PostAction::Delete(id)) => self.spawn_delete_post(ctx, &id),
                        Some(PostAction::StartEdit(id, title, content)) => {
                            self.editing_post_id = Some(id);
                            self.edit_title = title;
                            self.edit_content = content;
                        }
                        Some(PostAction::Update) => self.spawn_update_post(ctx),
                        Some(PostAction::CancelEdit) => {
                            self.editing_post_id = None;
                            self.edit_title.clear();
                            self.edit_content.clear();
                        }
                        None => {}
                    }
                }
            });
        });
    }
}
