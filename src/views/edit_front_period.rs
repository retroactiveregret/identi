use crate::api::local_naive_to_utc;
use crate::components::*;
use crate::icons::*;
use crate::models::*;
use crate::Route;
use chrono::Local;
use chrono::NaiveDate;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn EditFrontPeriod(id: Uuid) -> Element {
    let db = use_context::<Signal<Database>>();
    let mut status_message = use_context::<Signal<Status>>();
    let mut show_select = use_signal(|| false);

    let mut delete_warn = use_signal(|| false);

    let front_periods = (db().front_periods)();
    let fp = front_periods
        .get(&id)
        .cloned()
        .unwrap_or_else(|| FrontPeriod {
            id,
            started_at: Default::default(),
            ended_at: Default::default(),
            assignments: Default::default(),
        });

    let mut assignments = use_signal(|| fp.assignments.clone());
    let mut started_at = use_signal(|| fp.started_at.clone());
    let mut ended_at = use_signal(|| fp.ended_at.unwrap().clone());

    // Convert UTC -> Local for display/editing
    let mut start_date_part = use_signal(|| started_at().with_timezone(&Local).date_naive());
    let mut start_time_part = use_signal(|| started_at().with_timezone(&Local).time());

    let mut end_date_part = use_signal(|| ended_at().with_timezone(&Local).date_naive());
    let mut end_time_part = use_signal(|| ended_at().with_timezone(&Local).time());

    let save = move |_| {
        let start_naive = start_date_part().and_time(start_time_part());
        let end_naive = end_date_part().and_time(end_time_part());

        started_at.set(local_naive_to_utc(start_naive));
        ended_at.set(local_naive_to_utc(end_naive));

        match db().put_front_period(
            fp.id,
            started_at(),
            ended_at(),
            assignments(),
        ) {
            Ok(_) => {}
            Err(err) => status_message.write().set_message(
                format!("Error editing front period: {:#?}", err),
                StatusLevel::Error,
            ),
        }
        show_select.set(false);
        navigator().push(Route::Dashboard {});
    };

    let add_member = move |id: Uuid| {
        assignments.push(FrontPeriodAssignment {
            member_id: id,
            front_role: FrontRole::Unknown,
            confidence: 1.0,
            note: String::new(),
        });
        show_select.set(false);
    };

    let delete = move |_| {
        delete_warn.set(false);
        let mut binding = db();
        let mut write = binding.front_periods.write();
        write.shift_remove(&id).unwrap();
        navigator().go_back();
    };

    rsx! {
        div { class: "flex flex-col p-7 gap-4",
            div { class: "flex flex-row gap-4 overflow-x-scroll items-start",
                div { class: "h-24 w-24",
                    button {
                        class: "btn w-24 h-24 foreground grow rounded-box flex justify-center items-center p-0",
                        onclick: move |_| show_select.set(true),
                        Icon {
                            size: 64,
                            data: material_symbols_light::AddRounded,
                        }
                    }
                }
                Fronters { db, status_message, fp }
            }

            div { class: "flex flex-row gap-2 grow",
                div { class: "basis-1/2 flex flex-col gap-2 items-center p-2",
                    h2 { class: "text-2xl text-center", "Start time" }
                    input {
                        class: "w-fit",
                        r#type: "date",
                        value: start_date_part().format("%F").to_string(),
                        oninput: move |evt| start_date_part.set(evt.value().parse::<NaiveDate>().unwrap()),
                    }
                    input {
                        class: "w-fit",
                        r#type: "time",
                        value: start_time_part().format("%H:%M").to_string(),
                        oninput: move |evt| {
                            start_time_part
                                .set(chrono::NaiveTime::parse_from_str(&evt.value(), "%H:%M").unwrap());
                        },
                    }
                }

                div { class: "basis-1/2 flex flex-col gap-2 items-center p-2",
                    h2 { class: "text-2xl text-center", "End time" }
                    input {
                        class: "w-fit",
                        r#type: "date",
                        value: end_date_part().format("%F").to_string(),
                        oninput: move |evt| end_date_part.set(evt.value().parse::<NaiveDate>().unwrap()),
                    }
                    input {
                        class: "w-fit",
                        r#type: "time",
                        value: end_time_part().format("%H:%M").to_string(),
                        oninput: move |evt| {
                            end_time_part
                                .set(chrono::NaiveTime::parse_from_str(&evt.value(), "%H:%M").unwrap());
                        },
                    }
                }
            }

            div { class: "flex flex-row justify-between w-full",
                button {
                    class: "btn btn-error w-[5rem]",
                    onclick: move |_| delete_warn.set(true),
                    "Delete"
                }
                button { class: "btn btn-primary w-[5rem]", onclick: save, "Save" }
            }
        }

        MemberPicker { db, show_select, on_click: add_member }

        Modal { id: "delete-warn-modal", open: delete_warn,
            div { class: "modal-box",
                h3 { class: "text-lg font-bold", "Really delete switch event?" }
                p { class: "py-4", "This action cannot be undone." }
                div { class: "modal-action flex flex-row justify-between w-full",
                    button {
                        class: "btn",
                        onclick: move |_| delete_warn.set(false),
                        "Cancel"
                    }
                    button { class: "btn btn-error", onclick: delete, "Delete" }
                }
            }
        }
    }
}
