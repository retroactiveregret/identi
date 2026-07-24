use chrono::Utc;
use dioxus::prelude::*;
use std::{
    collections::HashSet,
    ops::{AddAssign, SubAssign},
};
use uuid::Uuid;

use crate::{
    components::{MemberAvatar, MemberPicker, Post},
    icons::*,
    models::*,
};

#[component]
pub fn BoardPosts(db: Signal<Database>, status_message: Signal<Status>) -> Element {
    let mut page = use_signal(|| 0);
    let max_show = 10;
    let board_len = use_memo(move || db().board_posts.read().len());

    let visible_posts =
        use_memo(move || db().get_unarchived_board_posts_paginated(max_show, page() * max_show));

    let mut author_id_input = use_signal(|| None);
    let mut mentions_input = use_signal(|| HashSet::<Uuid>::new());
    let mut content_input = use_signal(|| String::new());
    let mut pinned_input = use_signal(|| false);

    let create_post = move |_| {
        db().add_post(
            author_id_input(),
            mentions_input(),
            content_input(),
            pinned_input(),
            Utc::now(),
        );
        author_id_input.set(None);
        mentions_input.set(HashSet::<Uuid>::new());
        content_input.set(String::new());
        pinned_input.set(false);
    };

    rsx! {
        div { class: "w-full flex flex-col gap-2 p-2 pt-0",
            div { class: "p-4 pb-0 pt-0 small-heading", "Board" }
            div {
                EditBoardPost {
                    db,
                    status_message,
                    author_id_input,
                    mentions_input,
                    content_input,
                    pinned_input,
                }
                button {
                    class: "btn-sm btn btn-primary m-2 mt-1",
                    onclick: create_post,
                    "Post"
                }
            }
            for post_entry in visible_posts() {
                Post {
                    db,
                    status_message,
                    post: post_entry,
                    show_icons: true,
                }
            }
            div { class: "flex flex-row justify-between",
                div { class: "inline ml-2 mr-2",
                    if page() > 0 {
                        button {
                            class: "m-2 label text-xs",
                            aria_label: "Previous page",
                            onclick: move |_| page.sub_assign(1),
                            "Prev"
                        }
                    }
                    if (page() + 1) * max_show < board_len() {
                        button {
                            class: "m-2 label text-xs",
                            aria_label: "Next page",
                            onclick: move |_| page.add_assign(1),
                            "Next"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EditBoardPost(
    db: Signal<Database>,
    status_message: Signal<Status>,
    author_id_input: Signal<Option<Uuid>>,
    mentions_input: Signal<HashSet<Uuid>>,
    content_input: Signal<String>,
    pinned_input: Signal<bool>,
) -> Element {
    let mut show_author_select = use_signal(|| false);
    let mut show_mention_select = use_signal(|| false);

    let mentioned: Memo<Vec<Member>> = use_memo(move || {
        mentions_input()
            .iter()
            .map(|id| db().members.read().get(id).unwrap().to_owned())
            .collect()
    });
    let author_avatar = use_memo(move || match author_id_input() {
        Some(author) => Some(db().members.read().get(&author).unwrap().avatar_asset_id),
        None => None,
    });

    let set_author = move |id: Uuid| {
        author_id_input.set(Some(id));
        show_author_select.set(false);
    };
    let add_mention = move |id: Uuid| {
        mentions_input.insert(id);
        show_mention_select.set(false);
    };

    rsx! {
        div { class: "m-2",
            div { class: "textarea foreground w-full card-border",
                div { class: "mb-2 mt-1 flex flex-row justify-between w-full items-center",
                    button {
                        class: "btn btn-square rounded-box h-12 w-12",
                        aria_label: if author_avatar().is_some() { "Change post author" } else { "Select post author" },
                        onclick: move |_| show_author_select.set(true),
                        if let Some(avatar) = author_avatar() {
                            MemberAvatar { img_id: avatar, size: 12 }
                        } else {
                            Icon {
                                size: 32,
                                data: material_symbols_light::AddRounded,
                            }
                        }
                    }

                    div { class: "inline-flex",
                        Icon { class: "opacity-60", data: mdi_light::Pin }
                        input {
                            class: "toggle",
                            r#type: "checkbox",
                            value: pinned_input(),
                            oninput: move |evt| {
                                if evt.value().parse().unwrap_or(false) {
                                    pinned_input.set(true);
                                } else {
                                    pinned_input.set(false);
                                }
                            },
                        }
                    }
                }
                textarea {
                    class: "textarea-ghost resize-none w-full h-full",
                    placeholder: "Content",
                    value: content_input(),
                    oninput: move |evt| content_input.set(evt.value()),
                }
                div { class: "flex flex-row justify-between",
                    div { class: "flex flex-row gap-2 overflow-x-scroll grow",
                        for member in mentioned() {
                            div { class: "badge badge-primary", "@{member.name}" }
                        }
                    }
                    button {
                        class: "btn-circle",
                        aria_label: "Add mentioned member",
                        onclick: move |_| show_mention_select.set(true),
                        Icon { data: material_symbols_light::AddRounded }
                    }
                }
            }
        }

        MemberPicker { db, show_select: show_author_select, on_click: set_author }

        MemberPicker { db, show_select: show_mention_select, on_click: add_mention }
    }
}
