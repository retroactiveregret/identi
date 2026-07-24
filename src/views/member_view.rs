use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;
use crate::{Route, api::file_url, components::*, icons::*, models::*};

#[component]
pub fn MemberView(id: Uuid) -> Element {
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

    let custom_field_value_lookup = use_context::<Memo<CustomFieldValueLookup>>();
    let custom_field_values: HashMap<Uuid, String> = db()
        .custom_fields
        .read()
        .iter()
        .map(|(_, field)| {
            let initial_value = custom_field_value_lookup()
                .get(&(field.id, id))
                .cloned()
                .and_then(|value| match value.value {
                    Value::Text(s) => Some(s),
                    Value::Number(_) => None,
                    Value::Boolean(_) => None,
                })
                .unwrap_or_default();

            (field.id, initial_value)
        })
        .collect();

    rsx! {
        div { class: "",
            div { class: "top-0 mt-[-env(safe-area-inset-top)]",
                div { class: "w-screen h-48 p-0 m-0",
                    div { class: "flex w-screen h-full shadow-md bg-base-200",
                        match member.banner_asset_id {
                            Some(banner) => rsx! {
                                img { class: "w-full h-48 object-cover", src: file_url(banner) }
                            },
                            None => rsx! {},
                        }
                    }
                }
            }

            div { class: "m-7 mt-0",
                div { class: "w-full flex justify-center items-center",
                    div { class: "z-10 -mt-24",
                        MemberAvatar { img_id: member.avatar_asset_id, size: 48 }
                    }
                }
                div { class: "mt-7 w-full",
                    h1 { class: "text-center text-4xl font-bold", "{member.name}" }
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend opacity-60", "Description" }
                        Markdown {
                            class: "w-full prose",
                            text: member.description.clone(),
                        }

                        for (_ , field) in (db().custom_fields)()
                            .iter()
                            .filter(|(_, field)| !custom_field_values[&field.id].is_empty())
                        {
                            legend { class: "fieldset-legend opacity-60", "{field.name}" }
                            p { class: "w-full", "{custom_field_values[&field.id]}" }
                        }
                    }
                }
            }
        }

        div { class: "fab mb-[env(safe-area-inset-bottom)]",
            button { class: "btn w-12 h-12 p-0",
                Link { to: Route::EditMember { id },
                    Icon { size: 32, data: mdi_light::Pencil }
                }
            }
        }
    }
}
