use dioxus::prelude::*;
use crate::{Route, components::*, models::*};

#[component]
pub fn Fronters(
    db: Signal<Database>,
    status_message: Signal<Status>,
    fp: FrontPeriod,
) -> Element {
    let binding = db();
    let members = binding.members.read();

    let mut open_warning = use_signal(|| false);

    let fronters: Vec<Member> = fp.assignments
        .iter()
        .filter_map(|assignment| members.get(&assignment.member_id))
        .cloned()
        .collect();

    rsx! {
        for member in fronters {
            div { class: "", role: "button",
                Holdable {
                    onhold: move || open_warning.set(true),
                    onclick: move || {
                        navigator()
                            .push(Route::EditFrontAssignment {
                                event_id: fp.id,
                                member_id: member.id,
                            });
                    },
                    MemberAvatar { img_id: member.avatar_asset_id, size: 24 }
                }

                div { class: "w-24 flex flex-row justify-center",
                    label { class: "label text-center text-ellipsis", "{member.name}" }
                }
            }

            Modal { id: "remove-{member.id}", open: open_warning,
                div { class: "modal-box",
                    p { "Remove {member.name} from front?" }
                    div { class: "modal-action flex flex-row w-full justify-between",
                        form { class: "", method: "dialog",
                            button {
                                class: "btn",
                                onclick: move |_| open_warning.set(false),
                                "Cancel"
                            }
                        }
                        button {
                            class: "btn btn-error",
                            onclick: {
                                let fp = fp.clone();
                                move |_| {
                                    let mut assignments = fp.assignments.clone();
                                    assignments.retain(|a| a.member_id != member.id);
                                    match db().put_front_period(fp.id, fp.started_at, fp.ended_at, assignments) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            status_message
                                                .write()
                                                .set_message(
                                                    format!("Error removing fronter: {:#?}", e),
                                                    StatusLevel::Error,
                                                )
                                        }
                                    }
                                }
                            },
                            "Remove"
                        }
                    }
                }
            }
        }
    }
}
