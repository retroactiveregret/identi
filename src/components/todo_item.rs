use dioxus::prelude::*; 

use crate::models::*;

#[component]
pub fn TodoItem(task: TodoTask) -> Element {
    rsx! {
        div { class: "list-row",
            div {
                input { class: "checkbox btn-square", r#type: "checkbox" }
            }
            div { "{task.title}" }
        }
    }
}