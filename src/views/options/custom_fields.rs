use dioxus::prelude::*;
use uuid::Uuid;
use crate::{components::*, icons::*, models::*};

#[component]
pub fn CustomFields() -> Element {
    let db = use_context::<Signal<Database>>();

    let name_input = use_signal(|| String::new());

    let save_field = move |_| {
        let id = Uuid::new_v4();
        db().custom_fields.write().insert(id, CustomField { id, name: name_input(), field_type: FieldType::Text });
    };
    let delete_field = move |i: usize| {
        db().custom_fields.write().shift_remove_index(i);
    };
    
    rsx! {
        div {
            div { class: "p-4 pb-0 small-heading", "Custom Fields" }
            div { class: "w-full p-4",
                CustomFieldForm { db, name_input, on_click: save_field }
            }
            ul { class: "list w-full",
                for (i , (_ , field)) in (db().custom_fields)().iter().enumerate() {
                    li { class: "list-row w-full",
                        span { class: "list-col-grow", "{field.name}" }
                        label { class: "opacity-60", r#for: "delete_modal_{i}",
                            Icon {
                                size: 24,
                                data: material_symbols_light::DeleteOutlineRounded,
                            }
                        }
                    }
                }
            }
        }

        for (i , (_ , field)) in (db().custom_fields)().iter().enumerate() {
            input {
                class: "modal-toggle",
                id: "delete_modal_{i}",
                r#type: "checkbox",
            }
            div { class: "modal", role: "dialog",
                div { class: "modal-box",
                    h3 { class: "text-lg font-bold",
                        "Are you sure you want to delete custom field {field.name}?"
                    }
                    p { class: "py-4",
                        "This will make all data for {field.name} inaccessible. This action cannot be undone."
                    }
                    div { class: "modal-action",
                        label { class: "btn", r#for: "delete_modal_{i}", "Cancel" }
                        button {
                            class: "btn btn-error",
                            onclick: move |_| delete_field(i),
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}