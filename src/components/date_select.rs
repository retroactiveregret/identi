use chrono::NaiveDate;
use dioxus::prelude::*;
use crate::icons::*;

#[component]
pub fn DateSelect(mut date: Signal<NaiveDate>) -> Element {
    rsx! {
        div { class: "w-full p-5 foreground flex justify-between items-center",
            button {
                class: "btn-ghost",
                aria_label: "Previous date",
                onclick: move |_| date.set(date().pred_opt().unwrap()),
                Icon { data: material_symbols_light::ArrowBackIosRounded }
            }
            input {
                class: "text-xl font-semibold text-center",
                r#type: "date",
                value: date().format("%F").to_string(),
                oninput: move |evt| date.set(evt.value().parse::<NaiveDate>().unwrap()),
            }
            button {
                class: "btn-ghost",
                aria_label: "Next date",
                onclick: move |_| date.set(date().succ_opt().unwrap()),
                Icon { data: material_symbols_light::ArrowForwardIosRounded }
            }
        }
    }
}