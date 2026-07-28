#![allow(non_snake_case)]
#![allow(dead_code)]

mod api;
mod components;
mod db;
mod icons;
mod models;
mod views;

use chrono::NaiveDate;
use dioxus::{prelude::*, web::{Config, WebHistory}};
use std::{collections::HashMap, rc::Rc};
use uuid::Uuid;

use crate::{
    api::{eruda::*, request_persistent_storage},
    components::{NotificationPopup, StatusMessage},
    db::{load_database, save_database},
    icons::*,
    models::*,
    views::*,
};

fn main() {
    let history = WebHistory::new(
        Some("/identi".to_string()),
        true,
    );

    dioxus::LaunchBuilder::new()
        .with_cfg(Config::new().history(Rc::new(history)))
        .launch(App);
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Dashboard {},
        #[route("/members")]
        Members {},
        #[route("/journal")]
        Journal {},
        #[route("/notifications")]
        Notifications {},
        #[nest("/options")]
            #[route("/")]
            Options {},
            #[route("/appearance")]
            Appearance {},
            #[route("/appearance/theme")]
            Theme {},
            #[route("/custom_fields")]
            CustomFields {},
            #[route("/about")]
            About {},
            #[route("/board_history")]
            BoardHistory {},
            #[route("/front_history")]
            FrontHistory {},
            #[route("/import_export")]
            ImportExport {},
            #[route("/security")]
            Security {},
            #[route("/dev")]
            Dev {},
        #[end_nest]
    #[end_layout]
    #[layout(Fullscreen)]
        #[route("/members/new")]
        AddMember {},
        #[route("/members/:id")]
        MemberView { id: Uuid },
        #[route("/members/edit/:id")]
        EditMember { id: Uuid },
        #[route("/journal/post/:id")]
        JournalPost { id: Uuid },
        #[route("/journal/new")]
        AddJournalEntry {},
        #[route("/front_period/:id")]
        EditFrontPeriod { id: Uuid },
        #[route("/front_period/:event_id/:member_id")]
        EditFrontAssignment { event_id: Uuid, member_id: Uuid }
}

#[component]
fn App() -> Element {
    let mut ready: Signal<bool> = use_signal(|| false);
    let mut loaded_db: Signal<Option<DatabaseState>> = use_signal(|| None);

    use_hook(|| {
        spawn(async move {
            let loaded = load_database().await.unwrap();
            loaded_db.set(Some(loaded));
            ready.set(true);
        });
    });

    if ready() {
        rsx! {
            AppLoaded { loaded: loaded_db }
        }
    } else {
        rsx! {
            div { "Loading..." }
        }
    }
}

#[component]
fn AppLoaded(mut loaded: Signal<Option<DatabaseState>>) -> Element {
    let mut db = use_context_provider(|| {
        let initial = loaded
            .write()
            .take()
            .expect("AppLoaded is only rendered once `loaded` is Some");
        Signal::new(Database::from(initial))
    });
    let mut status_message = use_context_provider(|| Signal::new(Status::default()));

    let mut is_first_run = use_signal(|| true);
    use_effect(move || {
        let _ = db().members.read();
        let _ = db().taxonomy_terms.read();
        let _ = db().taxonomy_assignments.read();
        let _ = db().custom_fields.read();
        let _ = db().custom_field_values.read();
        let _ = db().todo_tasks.read();
        let _ = db().front_periods.read();
        let _ = db().journal_entries.read();
        let _ = db().board_posts.read();
        let _ = db().user_mentions.read();
        let _ = db().settings.read();

        if *is_first_run.peek() {
            is_first_run.set(false);
            return;
        }

        spawn(async move {
            match save_database(&DatabaseState::from(db())).await {
                Ok(_) => {}
                Err(e) => info!("{:#?}", e),
            }
        });
    });

    use_effect(move || {
        let document = web_sys::window().unwrap().document().unwrap();

        document
            .document_element()
            .unwrap()
            .set_attribute("data-theme", &(db().settings)().theme)
            .unwrap();
    });

    let user_mentions_lookup = use_memo(move || {
        let mut lookup = UserMentionsLookup(HashMap::<Uuid, Vec<Uuid>>::new());
        for (id, entry) in db().user_mentions.read().iter() {
            match lookup.get_mut(&entry.user_id) {
                Some(v) => v.push(*id),
                None => {
                    lookup.insert(entry.user_id, vec![*id]);
                }
            }
        }
        lookup
    });
    use_context_provider(|| user_mentions_lookup);

    let journal_dates = use_memo(move || {
        let mut journal_dates = JournalDates(HashMap::<NaiveDate, Vec<Uuid>>::new());
        for (id, entry) in db().journal_entries.read().iter() {
            match journal_dates.get_mut(&entry.created_at.date_naive()) {
                Some(v) => v.push(*id),
                None => {
                    journal_dates.insert(entry.created_at.date_naive(), vec![*id]);
                }
            }
        }
        journal_dates
    });
    use_context_provider(|| journal_dates);

    let switch_dates = use_memo(move || {
        let mut switch_dates = SwitchDates(HashMap::<NaiveDate, Vec<Uuid>>::new());
        for (id, entry) in db().front_periods.read().iter() {
            match switch_dates.get_mut(&entry.started_at.date_naive()) {
                Some(v) => v.push(*id),
                None => {
                    switch_dates.insert(entry.started_at.date_naive(), vec![*id]);
                }
            }
        }
        switch_dates
    });
    use_context_provider(|| switch_dates);

    let post_dates = use_memo(move || {
        let mut post_dates = PostDates(HashMap::<NaiveDate, Vec<Uuid>>::new());
        for (id, entry) in db().board_posts.read().iter() {
            match post_dates.get_mut(&entry.created_at.date_naive()) {
                Some(v) => v.push(*id),
                None => {
                    post_dates.insert(entry.created_at.date_naive(), vec![*id]);
                }
            }
        }
        post_dates
    });
    use_context_provider(|| post_dates);

    let custom_field_values_lookup = use_memo(move || {
        let mut lookup = CustomFieldValueLookup(HashMap::<(Uuid, Uuid), CustomFieldValue>::new());
        for (_, value) in &(db().custom_field_values)() {
            lookup.insert((value.field_id, value.member_id), value.clone());
        }
        lookup
    });
    use_context_provider(|| custom_field_values_lookup);

    let mut showed_notification = use_context_provider::<Signal<bool>>(|| Signal::new(false));
    use_effect(move || {
        let _ = (db().front_periods)();
        showed_notification.set(false);
    });

    let fronters_memo = use_memo(move || {
        let _ = (db().front_periods)();
        match db().get_active_period() {
            Some(fp) => Some(
                fp.assignments
                    .iter()
                    .map(|a| a.member_id)
                    .collect::<Vec<_>>(),
            ),
            None => None,
        }
    });
    use_context_provider(|| fronters_memo);

    spawn(async move {
        match request_persistent_storage().await {
            Ok(p) => {
                if !p {
                    status_message
                        .write()
                        .set_message("Persistent storage disabled", StatusLevel::Warning);
                }
            }
            Err(err) => status_message.write().set_message(
                format!("Error requesting persistent storage: {:#?}", err),
                StatusLevel::Error,
            ),
        }
    });

    use_effect(move || {
        if db().settings.read().dev_tools {
            eruda_init();
            eruda_set_button_visible(true);
        } else {
            eruda_set_button_visible(false);
        }
    });

    use_future(move || async move {
        loop {
            db.write().cleanup_todos();
            gloo_timers::future::sleep(std::time::Duration::from_secs(60)).await;
        }
    });

    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Navbar() -> Element {
    let status_message = use_context::<Signal<Status>>();
    let db = use_context::<Signal<Database>>();
    let user_mentions_lookup = use_context::<Memo<UserMentionsLookup>>();
    let fronters = use_context::<Memo<Option<Vec<Uuid>>>>();

    let unread = use_memo(move || {
        if let Some(members) = fronters() {
            (db().user_mentions)()
                .iter()
                .rev()
                .find(|&(_, n)| !n.read && members.contains(&n.user_id))
                .is_some()
        } else {
            false
        }
    });

    rsx! {
        StatusMessage { message: status_message }
        NotificationPopup { db, status_message, user_mentions_lookup }

        div { class: "pb-15 pt-[env(safe-area-inset-top)]", Outlet::<Route> {} }

        div { class: "dock foreground",
            span { class: "dock-label",
                Link { to: Route::Dashboard {},
                    Icon {
                        size: 32,
                        data: material_symbols_light::HomeOutlineRounded,
                    }
                }
                span { class: "dock-label", "Dashboard" }
            }
            span { class: "dock-label",
                Link { to: Route::Journal {},
                    Icon {
                        size: 32,
                        data: material_symbols_light::Book4OutlineRounded,
                    }
                }
                span { class: "dock-label", "Journal" }
            }
            span { class: "dock-label",
                Link { to: Route::Members {},
                    Icon {
                        size: 32,
                        data: material_symbols_light::UserAttributesOutlineRounded,
                    }
                }
                span { class: "dock-label", "Members" }
            }
            span { class: "dock-label",
                div { class: "relative",
                    Link { to: Route::Notifications {},
                        Icon {
                            size: 32,
                            data: material_symbols_light::NotificationsOutlineRounded,
                        }
                        if unread() {
                            div { class: "status status-primary absolute top-0 right-0" }
                        }
                    }
                }
                span { class: "dock-label", "Notifications" }
            }
            span { class: "dock-label",
                Link { to: Route::Options {},
                    Icon { size: 32, data: material_symbols_light::MenuRounded }
                }
                span { class: "dock-label", "Options" }
            }
        }
    }
}

#[component]
fn Fullscreen() -> Element {
    let status_message = use_context::<Signal<Status>>();
    let db = use_context::<Signal<Database>>();
    let user_mentions_lookup = use_context::<Memo<UserMentionsLookup>>();

    rsx! {
        StatusMessage { message: status_message }
        NotificationPopup { db, status_message, user_mentions_lookup }
        div { class: "pt-[env(safe-area-inset-top)]", Outlet::<Route> {} }
    }
}
