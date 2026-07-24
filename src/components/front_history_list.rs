use crate::{components::MemberAvatar, models::*, Route};
use chrono::Local;
use dioxus::prelude::*;
use indexmap::IndexMap;
use uuid::Uuid;

#[component]
pub fn FrontHistoryList(
    db: Signal<Database>,
    status_message: Signal<Status>,
    history: IndexMap<Uuid, FrontPeriod>,
    max_show: usize,
) -> Element {
    let mut history = (db().front_periods)();
    let active = db().get_active_period();
    if active.is_some() {
        history.pop();
    }
    let use_12h = true;
    let fmt_str = if use_12h { "%-I:%M %p" } else { "%R" };
    rsx! {
        for (_ , fp) in history.iter().rev().take(max_show) {
            li {
                class: "list-row flex overflow-x-scroll",
                onclick: {
                    let id = fp.id;
                    move |_| {
                        navigator().push(Route::EditFrontPeriod { id: id });
                    }
                },
                div {
                    label { class: "small-heading", "Start time" }
                    p { "{fp.started_at.with_timezone(&Local).format(\"%-m/%-d\")}" }
                    p { "{fp.started_at.with_timezone(&Local).format(fmt_str)}" }
                }
                div {
                    label { class: "small-heading", "End time" }
                    p { "{fp.ended_at.unwrap().with_timezone(&Local).format(\"%-m/%-d\")}" }
                    p { "{fp.ended_at.unwrap().with_timezone(&Local).format(fmt_str)}" }
                }
                for assignment in fp.assignments.clone() {
                    div {
                        MemberAvatar {
                            img_id: db().members.read()[&assignment.member_id].avatar_asset_id,
                            size: 24,
                        }
                    }
                }
            }
        }
    }
}
