use dioxus::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Modal(id: String, open: Signal<bool>, children: Element) -> Element {
    use_effect({
        let id = id.clone();
        move || {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            let dialog = document
                .get_element_by_id(&id)
                .unwrap()
                .dyn_into::<web_sys::HtmlDialogElement>()
                .unwrap();

            if open() {
                dialog.show_modal().unwrap();
            } else {
            }
        }
    });

    rsx! {
        dialog { class: "modal", id: "{id}",
            {children}
            form { class: "modal-backdrop", method: "dialog",
                button { "close" }
            }
        }
    }
}
