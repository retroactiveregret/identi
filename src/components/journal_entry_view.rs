use crate::{Route, components::{Markdown, MemberAvatar}, icons::*, models::*, api::*};
use chrono::Local;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn JournalEntryView(
    db: Signal<Database>,
    entry: JournalEntry,
    viewed_entry_id: Signal<Option<Uuid>>,
    date_entries: Memo<Option<Vec<Uuid>>>,
    viewed_position: Memo<Option<usize>>,
    show_post: Signal<bool>,
) -> Element {
    let members = (db().members)();
    let authors = entry.author_member_ids.iter().map(|id| members.get(id).unwrap()).collect::<Vec<_>>();

    let time = entry.created_at.with_timezone(&Local).time();
    let twelve_hour = (db().settings)().twelve_hour;

    rsx! {
        div { class: "m-4",
            div { class: "flex flex-row justify-between",
                button {
                    onclick: move |_| {
                        if let (Some(ids), Some(pos)) = (date_entries(), viewed_position()) {
                            if pos > 0 {
                                viewed_entry_id.set(ids.get(pos - 1).cloned());
                            }
                        }
                    },
                    class: if viewed_position().is_some_and(|p| p > 0) { "label" } else { "invisible" },
                    Icon { data: material_symbols_light::ArrowBackIosRounded }
                }

                div {
                    div { class: "flex flex-wrap gap-4 items-center",
                        div { class: "flex flex-row gap-4",
                            for member in authors {
                                MemberAvatar { img_id: member.avatar_asset_id, size: 12 }
                            }
                        }
                        if !entry.title.is_empty() {
                            h1 { class: "text-center text-xl font-semibold", "{entry.title}" }
                        } else {
                            h1 { class: "text-center text-xl label", "Untitled" }
                        }
                    }
                    div { class: "text-center w-full pt-2",
                        h2 { class: "label text-xs uppercase",
                            "{date_format(entry.created_at.date_naive())}, {time_format(time, twelve_hour)}"
                        }
                    }
                }

                button {
                    onclick: move |_| {
                        if let (Some(ids), Some(pos)) = (date_entries(), viewed_position()) {
                            if pos + 1 < ids.len() {
                                viewed_entry_id.set(ids.get(pos + 1).cloned());
                            }
                        }
                    },
                    class: if viewed_position().zip(date_entries()).is_some_and(|(p, ids)| p + 1 < ids.len()) { "label" } else { "invisible" },
                    Icon { data: material_symbols_light::ArrowForwardIosRounded }
                }
            }

            if entry.content_warning.is_some() && !show_post() {
                div { class: "flex justify-center items-center w-full h-[60vh]",
                    div { class: "foreground rounded-box p-5 w-full",
                        span { class: "inline-flex items-center gap-2 text-lg",
                            Icon {
                                class: "",
                                size: 32,
                                data: material_symbols_light::WarningOutlineRounded,
                            }

                            span { class: "text-xl font-semibold", "Content warning" }
                        }
                        p { class: "m-2", "{entry.content_warning.clone().unwrap()}" }
                        div { class: "flex justify-end w-full",
                            button {
                                class: "btn",
                                onclick: move |_| show_post.set(true),
                                "Show"
                            }
                        }
                    }
                }
            } else {
                div { class: "mt-7 flex flex-col",
                    Markdown { text: "{entry.body}", class: "prose grow" }
                }
            }
        }
        div {
            class: "fab bottom-34",
            tabindex: "0",
            role: "button",
            aria_label: "Edit entry",
            button { class: "btn btn-secondary w-12 h-12 p-0",
                Link { to: Route::JournalPost { id: entry.id },
                    Icon { size: 32, data: mdi_light::Pencil }
                }
            }
        }

        div {
            class: "fab bottom-20",
            tabindex: "0",
            role: "button",
            aria_label: "Add entry",
            button { class: "btn w-12 h-12 p-0",
                Link { to: Route::AddJournalEntry {},
                    Icon { size: 32, data: material_symbols_light::AddRounded }
                }
            }
        }
    }
}
