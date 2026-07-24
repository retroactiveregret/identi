use dioxus::prelude::*;
use crate::{models::Database};

#[component]
pub fn Security() -> Element {
    let db = use_context::<Signal<Database>>();
    let mut settings = db().settings;
    
    rsx! {
        div { class: "p-4 pb-0 small-heading", "Security" }
        ul { class: "list",
            li { class: "list-row gap-2",
                p { class: "", "Sanitize HTML" }
                div { class: "list-col-wrap ",
                    input {
                        class: "toggle",
                        r#type: "checkbox",
                        checked: settings().sanitize_html,
                        oninput: move |evt| {
                            if evt.value().parse().unwrap_or(false) {
                                settings.write().sanitize_html = true;
                            } else {
                                settings.write().sanitize_html = false;
                            }
                        },
                    }
                    if !settings().sanitize_html {
                        p { class: "text-error py-2",
                            "Disabling this can put your app at risk. "
                            label { class: "link", r#for: "warning-modal", "Learn more" }
                        }
                    }
                }
            }

            li { class: "list-row gap-2",
                p { class: "", "Enable developer tools" }
                div { class: "list-col-wrap ",
                    input {
                        class: "toggle",
                        r#type: "checkbox",
                        checked: settings().dev_tools,
                        oninput: move |evt| {
                            if evt.value().parse().unwrap_or(false) {
                                settings.write().dev_tools = true;
                            } else {
                                settings.write().dev_tools = false;
                            }
                        },
                    }
                }
            }
        }

        input { class: "modal-toggle", id: "warning-modal", r#type: "checkbox" }
        div { class: "modal", role: "dialog",
            div { class: "modal-box",
                h3 { class: "text-lg font-bold", "Warning" }
                p { class: "py-4",
                    "HTML sanitization prevents bad actors from posting templates that can compromize your app by interfering with your data (via. JavaScript) or making it unusable (via. CSS styling). However, these functions also allow more advanced stylistic control of the app."
                }
                p { class: "font-bold",
                    "Never paste templates into user descriptions or journal entries without understanding what they do if you choose to disable this."
                }

                div { class: "modal-action",
                    label { class: "btn", r#for: "warning-modal", "Close" }
                }
            }
            label { class: "modal-backdrop", r#for: "warning-modal", "Close" }
        }
    }
}