use dioxus::prelude::*;
use uuid::Uuid;

use crate::{components::{MemberAvatar, MemberPicker, Modal}, icons::*, models::*};

#[component]
pub fn JournalEntryEdit(
    db: Signal<Database>,
    title_input: Signal<String>,
    body_input: Signal<String>,
    authors_input: Signal<Vec<Uuid>>,
    content_warning_input: Signal<Option<String>>,
) -> Element {
    let content_warning_tmp = use_signal(|| content_warning_input().unwrap_or(String::new()));
    let checked = use_memo(move || content_warning_input().is_some());
    let mut show_select = use_signal(|| false);

    let add_author = move |id: Uuid| {
        authors_input.push(id);
        show_select.set(false);
    };

    rsx! {
        div { class: "flex flex-col gap-4 h-full grow",
            div { class: "flex flex-row justify-between",
                JournalEntryEditAuthorList { db, authors_input, show_select }
                JournalEntryEditContentWarning {
                    content_warning_input,
                    content_warning_tmp,
                    checked,
                }
            }
            JournalEntryEditTextFields { title_input, body_input }
        }

        MemberPicker { db, show_select, on_click: add_author }
    }
}

#[component]
fn JournalEntryEditAuthorList(
    db: Signal<Database>,
    authors_input: Signal<Vec<Uuid>>,
    show_select: Signal<bool>,
) -> Element {
    let authors = use_memo(move || {
        authors_input()
            .iter()
            .map(|id| db().members.read()[id].clone())
            .collect::<Vec<Member>>()
    });

    rsx! {
        div { class: "flex flex-row overflow-x-scroll gap-2",
            button {
                class: "btn w-12 h-12 foreground rounded-box flex justify-center items-center p-0",
                aria_label: "Add author",
                onclick: move |_| show_select.set(true),
                Icon { size: 24, data: material_symbols_light::AddRounded }
            }

            for member in authors() {
                button {
                    aria_label: "Remove author {member.name}",
                    onclick: move |_| {
                        authors_input
                            .set(
                                authors_input().iter().filter(|id| **id != member.id).cloned().collect(),
                            )
                    },
                    MemberAvatar { img_id: member.avatar_asset_id, size: 12 }
                }
            }
        }
    }
}

#[component]
fn JournalEntryEditTextFields(title_input: Signal<String>, body_input: Signal<String>) -> Element {
    rsx! {
        input {
            class: "input w-full",
            placeholder: "title",
            value: title_input(),
            oninput: move |evt| title_input.set(evt.value()),
        }
        textarea {
            class: "textarea w-full h-full grow",
            placeholder: "body",
            value: body_input(),
            oninput: move |evt| body_input.set(evt.value()),
        }
    }
}

#[component]
fn JournalEntryEditContentWarning(
    content_warning_input: Signal<Option<String>>,
    content_warning_tmp: Signal<String>,
    checked: Memo<bool>,
) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "flex flex-row h-12 btn btn-ghost",
            button {
                class: "flex items-center h-12",
                aria_label: "Toggle content warning",
                onclick: move |_| open.set(true),
                input {
                    class: "checkbox pointer-events-none",
                    r#type: "checkbox",
                    checked: checked(),
                }
                Icon {
                    size: 24,
                    data: material_symbols_light::WarningOutlineRounded,
                }
            }
        }
        Modal { id: "content-warning-modal", open,
            div { class: "modal-box",
                div { class: "py-4 flex flex-row justify-between",
                    span { "Add content warning to post?" }
                    input {
                        class: "toggle",
                        r#type: "checkbox",
                        value: "{checked()}",
                        oninput: move |evt| {
                            if evt.value().parse().unwrap_or(false) {
                                checked.set(true);
                            } else {
                                checked.set(false);
                                content_warning_input.set(None);
                            }
                        },
                    }
                }
                textarea {
                    class: "textarea mt-2",
                    placeholder: "Warning",
                    disabled: !checked(),
                    value: content_warning_tmp(),
                    oninput: move |evt| content_warning_tmp.set(evt.value()),
                }
                div { class: "flex flex-row justify-between mt-4",
                    button { class: "btn", onclick: move |_| open.set(false), "Cancel" }
                    button {
                        class: "btn",
                        onclick: move |_| {
                            open.set(false);
                            content_warning_input
                                .set(if checked() { Some(content_warning_tmp()) } else { None })
                        },
                        "Save"
                    }
                }
            }
        
        }
    }
}