use dioxus::{prelude::*, web::WebFileExt};
use std::collections::HashMap;
use uuid::Uuid;
use web_sys::Blob;

use crate::{
    api::{self, file_url},
    components::MemberAvatar,
    icons::*,
    models::*,
};

#[component]
pub fn MemberForm(
    db: Signal<Database>,
    name_input: Signal<String>,
    description_input: Signal<String>,
    avatar_id: Signal<Option<Uuid>>,
    banner_id: Signal<Option<Uuid>>,
    custom_field_inputs: HashMap<Uuid, Signal<String>>,
    archived_input: Option<Signal<bool>>,
) -> Element {
    let status_message = use_context::<Signal<Status>>();

    let on_image_avatar = move |uuid: Uuid| {
        avatar_id.set(Some(uuid));
    };
    let on_image_banner = move |uuid: Uuid| {
        banner_id.set(Some(uuid));
    };

    rsx! {
        div { class: "",
            div { class: "top-0 mt-[-env(safe-area-inset-top)]",
                ImageUpload {
                    on_image: on_image_banner,
                    uuid: banner_id,
                    status_message,
                    id: "banner-upload",
                    div { class: "w-screen h-48 p-0 m-0",
                        label {
                            class: "flex w-screen h-full shadow-md bg-base-200",
                            r#for: "banner-upload",
                            match banner_id() {
                                Some(banner) => rsx! {
                                    img { class: "w-full h-48 object-cover", src: file_url(banner) }
                                },
                                None => rsx! {
                                    Icon { class: "m-4", size: 24, data: mdi_light::Pencil }
                                },
                            }
                        }
                    }
                }
            }

            div { class: "m-7 mt-0",
                ImageUpload {
                    on_image: on_image_avatar,
                    uuid: avatar_id,
                    status_message,
                    id: "avatar-upload",
                    div { class: "w-full flex justify-center items-center",
                        label { class: "z-10 -mt-24", r#for: "avatar-upload",
                            MemberAvatar { img_id: avatar_id(), size: 48 }
                        }
                    }
                }

                div { class: "mt-7",
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend", "Name" }
                        input {
                            class: "input w-full",
                            r#type: "text",
                            placeholder: "Name",
                            value: "{name_input}",
                            oninput: move |event| name_input.set(event.value()),
                        }

                        legend { class: "fieldset-legend", "Description" }
                        textarea {
                            class: "textarea w-full",
                            placeholder: "Description",
                            value: "{description_input}",
                            oninput: move |event| description_input.set(event.value()),
                        }

                        for (_ , field) in (db().custom_fields)() {
                            legend { class: "fieldset-legend", "{field.name}" }
                            input {
                                class: "input w-full",
                                r#type: "text",
                                placeholder: "Name",
                                value: custom_field_inputs.get(&field.id).unwrap(),
                                oninput: {
                                    let mut inputs = custom_field_inputs.clone();
                                    move |event| {
                                        inputs.get_mut(&field.id).unwrap().set(event.value());
                                    }
                                },
                            }
                        }

                        match archived_input {
                            Some(mut archived) => rsx! {
                                legend { class: "fieldset-legend", "Archived" }
                                input {
                                    class: "toggle",
                                    r#type: "checkbox",
                                    value: archived(),
                                    oninput: move |evt| {
                                        if evt.value().parse().unwrap_or(false) {
                                            archived.set(true);
                                        } else {
                                            archived.set(false);
                                        }
                                    },
                                }
                            },
                            None => rsx! {},
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ImageUpload(
    on_image: Callback<Uuid>,
    uuid: Signal<Option<Uuid>>,
    status_message: Signal<Status>,
    id: String,
    children: Element,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        div { class: "image-upload",
            input {
                id,
                class: "hidden",
                r#type: "file",
                accept: "image/png,image/jpeg,image/webp,image/gif",

                onchange: move |evt| {
                    async move {
                        for file in evt.files() {
                            match file.get_web_file() {
                                Some(web_file) => {
                                    let blob: Blob = web_file.into();
                                    match api::upload(blob).await {
                                        Ok(new_uuid) => {
                                            on_image(new_uuid);
                                            uuid.set(Some(new_uuid));
                                            status_message
                                                .write()
                                                .set_message("Avatar uploaded.", StatusLevel::Success);
                                        }
                                        Err(e) => {
                                            status_message
                                                .write()
                                                .set_message(
                                                    format!("Upload failed: {e:?}"),
                                                    StatusLevel::Error,
                                                );
                                        }
                                    }
                                }
                                None => {
                                    error!("Failed to read image");
                                }
                            }
                        }
                    }
                },
            }

            {children}
        }
    }
}
