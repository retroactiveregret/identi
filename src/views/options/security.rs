use dioxus::prelude::*;
use crate::{components::Modal, models::Database};

#[component]
pub fn Security() -> Element {
    let db = use_context::<Signal<Database>>();
    let mut settings = db().settings;

    let mut open_warning = use_signal(|| false);
    
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
                            button {
                                class: "link",
                                onclick: move |_| open_warning.set(true),
                                "Learn more"
                            }
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

        Modal { id: "sanitization-warning-modal", open: open_warning,
            div { class: "modal-box",
                h3 { class: "text-lg font-bold", "Warning" }
                p { class: "pt-4",
                    "HTML sanitization prevents bad actors from posting templates that can compromize your app by interfering with your data (via. JavaScript) or making it unusable (via. CSS styling). However, these functions also allow more advanced stylistic control of the app."
                }
                p { class: "font-bold pt-4",
                    "Never paste templates into user descriptions or journal entries without understanding what they do if you choose to disable this."
                }
                p { class: "italic pt-4",
                    "Text entered with sanitization enabled will be unsanitized from this point forwards. Please review any untrusted HTML."
                }

                div { class: "modal-action",
                    button {
                        class: "btn",
                        onclick: move |_| open_warning.set(false),
                        "Close"
                    }
                }
            }
        }
    }
}