use chrono::Utc;
use dioxus::prelude::*;

use crate::models::*;

#[component]
pub fn TodoItem(db: Signal<Database>, task: TodoTask) -> Element {
    rsx! {
        div { class: "list-row",
            div { class: if task.completed_at.is_some() { "opacity-60" },
                input {
                    class: "checkbox rounded-square",
                    r#type: "checkbox",
                    aria_label: "Mark complete",
                    oninput: move |evt| {
                        if let Some(w) = db().todo_tasks.write().get_mut(&task.id) {
                            if evt.value().parse().unwrap_or(false) {
                                w.completed_at = Some(Utc::now());
                            } else {
                                w.completed_at = None
                            }
                        }
                    },
                }
            }
            div { class: if task.completed_at.is_some() { "opacity-60 line-through" },
                "{task.title}"
            }
        }
    }
}
