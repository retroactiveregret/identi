use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use dioxus::{logger::tracing::info, prelude::*};
use js_sys::Reflect;
use serde::Deserialize;
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Blob, Request, RequestInit, Response};

#[derive(Deserialize)]
struct ImageUploadResponse {
    id: String,
    #[allow(dead_code)]
    url: String,
}

pub async fn upload(blob: Blob) -> Result<Uuid, JsValue> {
    info!("Blob size {}", blob.size());
    let buffer = JsFuture::from(blob.array_buffer()).await?;

    let init = RequestInit::new();
    init.set_method("POST");
    init.set_body(&buffer);

    let request = Request::new_with_str_and_init("/files", &init)?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
    let fetch_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = fetch_value.dyn_into().unwrap_throw();

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "Image upload failed: {}",
            response.status()
        )));
    }

    let json_value = JsFuture::from(response.json()?).await?;
    let id_value = Reflect::get(&json_value, &JsValue::from_str("id"))?;
    let id_str = id_value
        .as_string()
        .ok_or_else(|| JsValue::from_str("Missing image id"))?;

    Uuid::parse_str(&id_str).map_err(|e| JsValue::from_str(&format!("Invalid image id: {e}")))
}

pub fn file_url(id: Uuid) -> String {
    format!("/files/{id}")
}

fn not_rand(start: usize, end: usize, seed: usize) -> usize {
    (start + seed) % end
}

pub fn local_naive_to_utc(naive: NaiveDateTime) -> DateTime<Utc> {
    match Local.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt.with_timezone(&Utc),
        chrono::LocalResult::Ambiguous(earliest, _latest) => earliest.with_timezone(&Utc),
        chrono::LocalResult::None => {
            let mut adjusted = naive;
            loop {
                adjusted += chrono::Duration::hours(1);
                if let chrono::LocalResult::Single(dt) = Local.from_local_datetime(&adjusted) {
                    break dt.with_timezone(&Utc);
                }
            }
        }
    }
}

pub fn time_format(time: NaiveTime, twelve_hour: bool) -> String {
    if twelve_hour {
        time.format("%I:%M %P").to_string()
    } else {
        time.format("%H:%M").to_string()
    }
}

pub fn date_format(date: NaiveDate) -> String {
    date.format("%x").to_string()
}

pub async fn request_persistent_storage() -> Result<bool, wasm_bindgen::JsValue> {
    let storage = window().unwrap().navigator().storage();

    let persisted = JsFuture::from(storage.persisted()?).await?;
    if persisted.as_bool() == Some(true) {
        return Ok(true);
    }

    let granted = JsFuture::from(storage.persist()?).await?;
    Ok(granted.as_bool().unwrap_or(false))
}

pub mod eruda {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(inline_js = r#"
        export function eruda_init() {
            if (window.eruda) { window.eruda.init(); }
        }
        export function eruda_destroy() {
            try { window.eruda.destroy(); } catch (e) {}
        }
        export function eruda_show() {
            try { window.eruda.show(); } catch (e) {}
        }
        export function eruda_set_button_visible(visible) {
            const btn = document.querySelector('.eruda-entry-btn');
            if (btn) { btn.style.display = visible ? '' : 'none'; }
        }
    "#)]
    extern "C" {
        pub fn eruda_init();
        pub fn eruda_destroy();
        pub fn eruda_show();
        pub fn eruda_set_button_visible(visible: bool);
    }
}
