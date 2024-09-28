use crate::utils::{auth, config::CONFIG, secret::SECRET};
use relm4::{adw::prelude::*, prelude::*};

pub struct LoginPage {
    username_entry: adw::EntryRow,
    password_entry: adw::PasswordEntryRow,
    save_password_switch: adw::SwitchRow,
    is_processing: bool,
}

#[derive(Debug)]
pub enum LoginPageInput {
    Submit,
    ToggleSavePassword,
}

#[derive(Debug)]
pub enum LoginPageOutput {
    LoggedIn,
}

#[derive(Debug)]
pub enum LoginPageMessage {
    LoggedIn,
    Unauthorized,
    Error(String),
}

#[relm4::component(pub)]
impl Component for LoginPage {
    type Init = ();
    type Input = LoginPageInput;
    type Output = LoginPageOutput;
    type CommandOutput = LoginPageMessage;

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {},

            #[wrap(Some)]
            set_content = &adw::Clamp {
                gtk::ListBox {
                    set_valign: gtk::Align::Center,
                    #[local_ref]
                    username_entry -> adw::EntryRow {
                        set_title: "Nom d'utilisateur",
                        connect_activate => LoginPageInput::Submit,
                    },
                    #[local_ref]
                    password_entry -> adw::PasswordEntryRow {
                        set_title: "Mot de passe",
                        connect_activate => LoginPageInput::Submit,
                    },
                    #[local_ref]
                    save_password_switch -> adw::SwitchRow {
                        set_title: "Sauvegarder le mot de passe",
                        set_active: CONFIG.read().unwrap().save_credentials(),
                        connect_active_notify => LoginPageInput::ToggleSavePassword,
                    },
                    gtk::ListBoxRow {
                        gtk::Button {
                            set_label: "Se connecter",
                            #[watch]
                            set_sensitive: !model.is_processing,
                            add_css_class: "suggested-action",
                            connect_clicked => LoginPageInput::Submit,
                        },
                    },
                },
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            username_entry: adw::EntryRow::default(),
            password_entry: adw::PasswordEntryRow::default(),
            save_password_switch: adw::SwitchRow::default(),
            is_processing: false,
        };

        let username_entry = model.username_entry.clone();
        let password_entry = model.password_entry.clone();
        let save_password_switch = model.save_password_switch.clone();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            LoginPageInput::ToggleSavePassword => {
                let is_active = self.save_password_switch.is_active();
                if !is_active {
                    gtk::glib::spawn_future_local(SECRET.remove_auth());
                }
                CONFIG.write().unwrap().set_save_credentials(is_active);
            }
            LoginPageInput::Submit => {
                self.is_processing = true;

                let username = self.username_entry.text();
                let password = self.password_entry.text();

                gtk::glib::spawn_future_local(async move {
                    let auth_result = auth::login(username.into(), password.into()).await;
                    let command = match auth_result {
                        Ok(_) => LoginPageMessage::LoggedIn,
                        Err(auth::Error::BadCredentials) => LoginPageMessage::Unauthorized,
                        Err(_) => LoginPageMessage::Error("An error occurred".into()),
                    };
                    sender
                        .command_sender()
                        .send(command)
                        .expect("failed to send command");
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            LoginPageMessage::LoggedIn => {
                sender
                    .output(LoginPageOutput::LoggedIn)
                    .expect("failed to send login signal");
            }
            LoginPageMessage::Unauthorized => {
                println!("Bad credentials");
            }
            LoginPageMessage::Error(error) => {
                eprintln!("Error: {}", error);
            }
        }
        self.is_processing = false;
    }
}
