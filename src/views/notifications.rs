use crate::{components::*, models::*};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn Notifications() -> Element {
    let db = use_context::<Signal<Database>>();
    let outline = (db().settings)().outline_notifications;
    let mark_read = move |id: Uuid| db().mark_notification_read(id, true).unwrap();
    let mark_all_read = move |_| db().mark_all_notifications_read();
    rsx! {
        div {
            div { class: "p-4 small-heading", "Notifications" }
            div { class: "p-4",
                button { class: "btn-sm btn btn-primary", onclick: mark_all_read, "Mark all read" }
            }
            ul { class: "list foreground rounded-xl shadow-md overflow-hidden m-4 mt-0",
                {
                    let current_db = db();
                    rsx! {
                        for (_ , notification) in current_db.user_mentions.read().iter().rev() {
                            {
                                let notification_id = notification.id;
                                let user_id = notification.user_id;
                                let board_post_id = notification.board_post_id;
                                let is_read = notification.read;

                                rsx! {
                                    li {
                                        class: format!(
                                            "list-row rounded-none overflow-hidden {}",
                                            if !is_read && !outline { "bg-secondary text-secondary-content" } else { "" },
                                        ),
                                        onclick: move |_| mark_read(notification_id),
                                        div {
                                            class: format!(
                                                "rounded-box {}",
                                                if !is_read && outline {
                                                    "outline-4 outline-offset-3 outline-solid outline-secondary"
                                                } else {
                                                    ""
                                                },
                                            ),
                                            MemberAvatar { img_id: db().members.read()[&user_id].avatar_asset_id, size: 16 }
                                        }
                                        p { class: "line-clamp-3", "{db().board_posts.read()[&board_post_id].content}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
