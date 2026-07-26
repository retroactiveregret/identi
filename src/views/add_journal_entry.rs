use crate::components::*;
use crate::models::*;
use crate::Route;
use chrono::Utc;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn AddJournalEntry() -> Element {
    let db = use_context::<Signal<Database>>();
    let mut status_message = use_context::<Signal<Status>>();

    let title_input = use_signal(|| String::new());
    let body_input = use_signal(|| String::new());
    let authors_input = use_signal(|| Vec::<Uuid>::new());
    let content_warning_input = use_signal(|| None);

    let save_post = move |_| {
        if !body_input.is_empty() {
            db().add_journal_entry(
                title_input(),
                body_input(),
                Utc::now(),
                authors_input(),
                content_warning_input(),
            );
            navigator().push(Route::Journal {});
        } else {
            status_message
                .write()
                .set_message("Please add content to the post", StatusLevel::Error);
        }
    };

    rsx! {

        div { class: "min-h-screen flex flex-col p-7 gap-4",
            JournalEntryEdit {
                db,
                title_input,
                body_input,
                authors_input,
                content_warning_input,
            }

            div { class: "flex flex-row justify-between",
                button { class: "btn", onclick: move |_| navigator().go_back(), "Cancel" }
                button { class: "btn btn-primary", onclick: save_post, "Save" }
            }
        }
    }
}
