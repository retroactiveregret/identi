use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{components::*, models::*, Route};

#[component]
pub fn AddMember() -> Element {
    let db = use_context::<Signal<Database>>();
    let name_input = use_signal(|| String::new());
    let description_input = use_signal(|| String::new());
    let avatar_id = use_signal(|| Option::<uuid::Uuid>::None);
    let banner_id = use_signal(|| Option::<uuid::Uuid>::None);

    let custom_field_inputs: HashMap<uuid::Uuid, Signal<String>> = db()
        .custom_fields
        .read()
        .iter()
        .map(|(_, field)| (field.id, use_signal(|| String::new())))
        .collect();

    let mut status_message = use_context::<Signal<Status>>();

    let create_member = {
        let custom_field_inputs = custom_field_inputs.clone();
        move |_| {
            let custom_field_inputs = custom_field_inputs.clone();
            let name = name_input().trim().to_string();
            let description = description_input().trim().to_string();

            if name.is_empty() {
                status_message.write().set_message(
                    "Please enter a name before creating a member.",
                    StatusLevel::Error,
                );
                return;
            }

            spawn(async move {
                let id = db().add_member(name, description, None, avatar_id(), banner_id());
                let custom: Vec<CustomFieldValue> = custom_field_inputs
                    .iter()
                    .map(|(f, s)| CustomFieldValue {
                        field_id: *f,
                        member_id: id,
                        value: Value::Text(s().to_string()),
                    })
                    .collect();
                db().add_custom_field_values(custom);
                navigator().push(Route::Members {});
                status_message
                    .write()
                    .set_message("Member created successfully.", StatusLevel::Success);
            });
        }
    };

    rsx! {
        div {
            MemberForm {
                db,
                name_input,
                description_input,
                avatar_id,
                banner_id,
                custom_field_inputs,
            }

            div { class: "p-7 flex flex-row justify-between w-full",
                button { class: "btn", onclick: move |_| navigator().go_back(), "Cancel" }
                button { class: "btn btn-primary", onclick: create_member, "Create member" }
            }
        }
    }
}
