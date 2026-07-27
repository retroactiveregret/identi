use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::{
    components::*,
    models::{self, *},
    Route,
};

#[component]
pub fn EditMember(id: Uuid) -> Element {
    let db = use_context::<Signal<Database>>();
    let mut status_message = use_context::<Signal<Status>>();

    let members = db().members;
    let binding = members.read();
    let member = binding.get(&id);
    if member.is_none() {
        navigator().push(Route::Members {});
        status_message.write().set_message(
            format!("Unable to find member with ID {}", id),
            StatusLevel::Error,
        );
        return rsx! {};
    }
    let member = member.unwrap();

    let name_input = use_signal(|| member.name.to_owned());
    let description_input = use_signal(|| member.description.to_owned());
    let avatar_id = use_signal(|| member.avatar_asset_id);
    let banner_id = use_signal(|| member.banner_asset_id);
    let archived_input = use_signal(|| member.archived);

    let custom_field_values = use_context::<Memo<CustomFieldValueLookup>>();
    let custom_field_inputs: HashMap<Uuid, Signal<String>> = db()
        .custom_fields
        .read()
        .iter()
        .map(|(_, field)| {
            let initial_value = custom_field_values()
                .get(&(field.id, id))
                .cloned()
                .and_then(|value| match value.value {
                    Value::Text(s) => Some(s),
                    Value::Number(_) => None,
                    Value::Boolean(_) => None,
                })
                .unwrap_or_default();

            (field.id, use_signal(|| initial_value))
        })
        .collect();

    let mut status_message = use_context::<Signal<Status>>();

    let save_member = {
        let member = member.clone();
        let custom_field_inputs = custom_field_inputs.clone();

        move |_| {
            let name = name_input().trim().to_string();
            let description = description_input().trim().to_string();
            let avatar = avatar_id();
            let banner = banner_id();

            if name.is_empty() {
                status_message.write().set_message(
                    "Please enter a name before creating a member.",
                    StatusLevel::Error,
                );
                return;
            }

            let edited_member = Member {
                id,
                name,
                description,
                color: member.color,
                avatar_asset_id: avatar,
                banner_asset_id: banner,
                archived: archived_input(),
                created_at: member.created_at,
            };

            let values = custom_field_inputs
                .iter()
                .map(|(&field_id, sig)| CustomFieldValue {
                    field_id,
                    member_id: id,
                    value: models::Value::Text(sig()),
                })
                .collect::<Vec<_>>();

            match db().put_member(&edited_member) {
                Ok(_) => {
                    db().add_custom_field_values(values);
                    status_message
                        .write()
                        .set_message("Updated member successfully", StatusLevel::Success);
                    navigator().push(Route::Members {});
                }
                Err(err) => status_message.write().set_message(
                    format!("Failed to create member: {err:?}"),
                    StatusLevel::Error,
                ),
            }
        }
    };

    rsx! {
        div { class: "",
            MemberForm {
                db,
                name_input,
                description_input,
                avatar_id,
                banner_id,
                custom_field_inputs,
                archived_input,
            }

            div { class: "p-7 pt-0 flex flex-row w-full justify-between",
                button { class: "btn", onclick: move |_| navigator().go_back(), "Cancel" }
                button { class: "btn btn-primary", onclick: save_member, "Save member" }
            }
        }
    }
}
