use dioxus::prelude::*;
use crate::{components::*, models::*};

#[component]
pub fn ImportExport() -> Element {
    let db = use_context::<Signal<Database>>();
    let status_message = use_context::<Signal<Status>>();

    rsx! {
        div { class: "p-4 pb-0 small-heading", "Importing and Exporting" }
        ul { class: "list",
            li { class: "list-row",
                DownloadButton { db, status_message }
            }
            li { class: "list-row",
                UploadButton { db, status_message }
            }
        }
    }
}