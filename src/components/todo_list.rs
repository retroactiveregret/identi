use dioxus::prelude::*;

use crate::{components::*, icons::*, models::*};

#[component]
pub fn TodoList(db: Signal<Database>) -> Element {
    let tasks = use_memo(move || (db().todo_tasks)());
    let mut open_daily = use_signal(|| false);

    rsx! {
        div { class: "w-full flex flex-col gap-2 p-2 pt-0",
            div { class: "p-4 pb-0 pt-0 small-heading", "Todo" }
            div { class: "p-4 m-2 mt-0 rounded-box foreground",
                ul { class: "list",
                    div { class: "p-4 pb-0 pt-0 flex flex-row justify-between items-center",
                        span { class: "small-heading", "Daily" }
                        button { onclick: move |_| open_daily.set(true),
                            Icon {
                                size: 24,
                                data: material_symbols_light::AddRounded,
                            }
                        }
                    }
                    for (_ , task) in tasks().iter().filter(|(_, t)| t.todo_type == TodoType::Daily) {
                        TodoItem { task: task.clone() }
                    }
                    li { class: "list-row hidden" }
                }
                div { class: "p-4 pb-0 pt-0 flex flex-row justify-between items-center",
                    span { class: "small-heading", "One-off" }
                    button {
                        Icon {
                            size: 24,
                            data: material_symbols_light::AddRounded,
                        }
                    }
                }
                ul { class: "list",
                    for (_ , task) in tasks().iter().filter(|(_, t)| t.todo_type == TodoType::Single) {
                        TodoItem { task: task.clone() }
                    }
                }
            }
        }

        Modal { id: "daily", open: open_daily,
            div { class: "modal-box",
                h1 { "meow" }
                form { method: "dialog",
                    button { class: "btn", "Close" }
                }
            }
        }
    }
}