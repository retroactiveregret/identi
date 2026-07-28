use dioxus::prelude::*;

use crate::{components::*, icons::*, models::*};

#[component]
pub fn TodoList(db: Signal<Database>) -> Element {
    let tasks = use_memo(move || (db().todo_tasks)());
    let mut open_daily = use_signal(|| false);
    let mut open_single = use_signal(|| false);

    let mut daily_value = use_signal(|| String::new());
    let mut single_value = use_signal(|| String::new());

    rsx! {
        div { class: "w-full flex flex-col gap-2 p-2 pt-0",
            div { class: "p-4 pb-0 pt-0 small-heading", "To-do" }
            div { class: "p-4 m-2 mt-0 rounded-box foreground",
                ul { class: "list",
                    div { class: "p-4 pb-0 pt-0 flex flex-row justify-between items-center",
                        span { class: "small-heading", "Daily" }
                        button {
                            aria_label: "Add daily task",
                            onclick: move |_| open_daily.set(true),
                            Icon {
                                size: 24,
                                data: material_symbols_light::AddRounded,
                            }
                        }
                    }
                    for (_ , task) in tasks().iter().filter(|(_, t)| t.todo_type == TodoType::Daily) {
                        TodoItem { db, task: task.clone() }
                    }
                }
                ul { class: "list",
                    div { class: "p-4 pb-0 pt-0 flex flex-row justify-between items-center",
                        span { class: "small-heading", "One-off" }
                        button {
                            aria_label: "Add one-off task",
                            onclick: move |_| open_single.set(true),
                            Icon {
                                size: 24,
                                data: material_symbols_light::AddRounded,
                            }
                        }
                    }
                    for (_ , task) in tasks().iter().filter(|(_, t)| t.todo_type == TodoType::Single) {
                        TodoItem { db, task: task.clone() }
                    }
                }
            }
        }

        Modal { id: "daily", open: open_daily,
            div { class: "modal-box pt-2",
                div { class: "flex flex-row items-center mt-0 pb-1",
                    span { class: "small-heading grow", "Add daily task" }
                    form { method: "dialog",
                        button { class: "btn btn-sm btn-circle btn-ghost text-base-content/60",
                            Icon {
                                size: 24,
                                data: material_symbols_light::CloseRounded,
                            }
                        }
                    }
                }
                div { class: "flex flex-row gap-2",
                    input {
                        class: "input grow",
                        placeholder: "Todo text",
                        value: daily_value(),
                        oninput: move |evt| daily_value.set(evt.value()),
                    }
                    button {
                        class: "btn btn-primary btn-square",
                        onclick: move |_| {
                            db().add_todo_task(daily_value(), TodoType::Daily);
                            open_daily.set(false);
                        },
                        Icon {
                            size: 32,
                            data: material_symbols_light::AddRounded,
                        }
                    }
                }
            }
        }

        Modal { id: "single", open: open_single,
            div { class: "modal-box pt-2",
                div { class: "flex flex-row items-center mt-0 pb-1",
                    span { class: "small-heading grow", "Add one-off task" }
                    form { method: "dialog",
                        button { class: "btn btn-sm btn-circle btn-ghost text-base-content/60",
                            Icon {
                                size: 24,
                                data: material_symbols_light::CloseRounded,
                            }
                        }
                    }
                }
                div { class: "flex flex-row gap-2",
                    input {
                        class: "input grow",
                        placeholder: "Todo text",
                        value: single_value(),
                        oninput: move |evt| single_value.set(evt.value()),
                    }
                    button {
                        class: "btn btn-primary btn-square",
                        onclick: move |_| {
                            db().add_todo_task(single_value(), TodoType::Single);
                            open_single.set(false);
                        },
                        Icon {
                            size: 32,
                            data: material_symbols_light::AddRounded,
                        }
                    }
                }
            }
        }
    }
}