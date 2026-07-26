use crate::{components::*, models::*};
use chrono::Utc;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn EditFrontAssignment(event_id: Uuid, member_id: Uuid) -> Element {
    let db = use_context::<Signal<Database>>();
    let front_periods = (db().front_periods)();
    let members = (db().members)();

    let fp = front_periods.get(&event_id).unwrap();
    let assignments = fp.to_owned().assignments;
    let assignment = assignments
        .iter()
        .find(|&a| a.member_id == member_id)
        .unwrap()
        .to_owned();
    let member = members.get(&member_id).unwrap();

    let front_role = use_signal(|| assignment.front_role);
    let mut confidence = use_signal(|| assignment.confidence);
    let mut note = use_signal(|| assignment.note);

    let create = move |_| {
        let mut assignments_new = assignments.clone();
        for assignment in assignments_new.iter_mut() {
            if assignment.member_id == member_id {
                assignment.front_role = front_role();
                assignment.confidence = confidence();
                assignment.note = note();
            }
        }
        db().switch(Utc::now(), assignments_new);
        navigator().go_back();
    };

    let update = move |_| {
        let mut binding = db();
        let mut write = binding.front_periods.write();
        let fp_mut = write.get_mut(&event_id).unwrap();
        let mut assignments_new = fp_mut.assignments.clone();

        for assignment in assignments_new.iter_mut() {
            if assignment.member_id == member_id {
                assignment.front_role = front_role();
                assignment.confidence = confidence();
                assignment.note = note();
            }
        }

        fp_mut.assignments = assignments_new;
        navigator().go_back();
    };

    rsx! {
        div { class: "p-7",
            div { class: "w-full flex justify-center",
                MemberAvatar { img_id: member.avatar_asset_id, size: 48 }
            }

            div { class: "grid grid-cols-2 gap-4",
                fieldset { class: "fieldset w-full mt-7",
                    legend { class: "fieldset-legend", "Front Role" }
                    FrontRoleDropdown { front_role }
                }
                fieldset { class: "fieldset w-full mt-7",
                    legend { class: "fieldset-legend", "Confidence" }
                    input {
                        class: "input w-full",
                        value: "{confidence()}",
                        oninput: move |evt| {
                            match evt
                                .value()
                                .chars()
                                .filter(|c| c.is_ascii_digit() || *c == '.')
                                .collect::<String>()
                                .parse::<f64>()
                            {
                                Ok(c) => confidence.set(c),
                                Err(_) => {}
                            }
                        },
                    }
                }
            }

            fieldset { class: "fieldset w-full mt-4",
                legend { class: "fieldset-legend", "Note" }
                textarea {
                    class: "textarea w-full",
                    placeholder: "Assignment note",
                    value: "{note()}",
                    oninput: move |evt| note.set(evt.value()),
                }
            }

            div { class: "w-full mt-4 flex flex-row justify-between",
                button { class: "btn btn btn-primary", onclick: update, "Update Period" }
                button { class: "btn btn-primary", onclick: create, "Create Period" }
            }
        }
    }
}
