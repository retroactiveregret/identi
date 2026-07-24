use crate::{Route, components::*, models::*};
use chrono::Utc;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn JournalPost(id: Uuid) -> Element {
    let db = use_context::<Signal<Database>>();
    let mut status_message = use_context::<Signal<Status>>();
    let binding = (db().journal_entries)();
    let post = binding.get(&id).unwrap();

    let title_input = use_signal(|| post.title.clone());
    let body_input = use_signal(|| post.body.clone());
    let authors_input = use_signal(|| post.author_member_ids.clone());
    let content_warning_input = use_signal(|| post.content_warning.clone());

    let save_post = move |_| match db().put_journal_entry(
        id,
        title_input(),
        body_input(),
        Utc::now(),
        authors_input(),
        content_warning_input(),
    ) {
        Ok(post) => {
            info!("Created journal post {:#?}", post);
            navigator().push(Route::Journal {});
        }
        Err(e) => status_message
            .write()
            .set_message(format!("Error creating post: {:#?}", e), StatusLevel::Error),
    };

    rsx! {
        div { class: "min-h-screen flex flex-col gap-4",
            div { class: "grow h-full",
                JournalEntryEdit {
                    db,
                    title_input,
                    body_input,
                    authors_input,
                    content_warning_input,
                }
            }

            div { class: "flex flex-row justify-between m-7",
                button { class: "btn", onclick: move |_| navigator().go_back(), "Cancel" }
                button { class: "btn btn-primary", onclick: save_post, "Save" }
            }
        }
    }
}
