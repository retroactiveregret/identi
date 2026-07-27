use dioxus::prelude::*;
use crate::{components::*, icons::*, models::*};

#[component]
pub fn Post(db: Signal<Database>, status_message: Signal<Status>, post: BoardPost, show_icons: bool) -> Element {
    let mentioned = post
        .mentions
        .iter()
        .map(|id| db().members.read()[id].clone());

    let archive_post = move |_| match db().archive_post(post.id, true) {
        Ok(_) => {}
        Err(err) => status_message.write().set_message(
            format!("Error archiving post: {:#?}", err),
            StatusLevel::Error,
        ),
    };

    let author_avatar = if let Some(id) = post.author_id {
        db().members.read().get(&id).unwrap().avatar_asset_id
    } else {
        None
    };

    rsx! {
        div { class: "foreground p-5 rounded-box flex flex-col gap-2 m-2",
            div { class: "flex flex-row justify-between",
                if let Some(_) = post.author_id {
                    MemberAvatar { img_id: author_avatar, size: 12 }
                } else {
                    div {}
                }
                if post.pinned && show_icons {
                    Icon { class: "opacity-60", data: mdi_light::Pin, size: 24 }
                } else {
                    div {}
                }
            }

            Markdown { class: "prose", text: post.content }

            div { class: "flex flex-row mt-2",
                div { class: "flex flex-row gap-2 overflow-x-scroll grow",
                    for member in mentioned {
                        div { class: "badge badge-primary", "@{member.name}" }
                    }
                }
                if show_icons {
                    button { aria_label: "Archive post", onclick: archive_post,
                        Icon {
                            class: "opacity-60",
                            data: material_symbols_light::ArchiveOutlineRounded,
                        }
                    }
                }
            }
        }
    }
}