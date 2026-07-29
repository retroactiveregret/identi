use dioxus::logger::tracing::info;
use js_sys::Promise;
use serde::Serialize;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransaction, IdbTransactionMode};

use crate::models::DatabaseState;

const DB_NAME: &str = "dioxus_app_db";

const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "app_store";

const DB_KEY: &str = "root";

fn open_request_to_promise(request: IdbOpenDbRequest) -> Promise {
    Promise::new(&mut |resolve, reject| {
        {
            let req = request.clone();
            let cb = Closure::<dyn FnMut(web_sys::IdbVersionChangeEvent)>::new(
                move |_: web_sys::IdbVersionChangeEvent| {
                    let db: IdbDatabase = req
                        .result()
                        .expect("onupgradeneeded: missing result")
                        .dyn_into()
                        .expect("onupgradeneeded: result is not IdbDatabase");

                    let names = db.object_store_names();
                    let exists =
                        (0..names.length()).any(|i| names.get(i).as_deref() == Some(STORE_NAME));

                    if !exists {
                        db.create_object_store(STORE_NAME)
                            .expect("Failed to create object store");
                    }
                },
            );
            request.set_onupgradeneeded(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        {
            let req = request.clone();
            let resolve = resolve.clone();
            let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                let db = req.result().unwrap_or(JsValue::UNDEFINED);
                resolve.call1(&JsValue::NULL, &db).unwrap();
            });
            request.set_onsuccess(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        {
            let req = request.clone();
            let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                let err = req
                    .error()
                    .ok()
                    .flatten()
                    .map(Into::into)
                    .unwrap_or_else(|| JsValue::from_str("IDB open: unknown error"));
                reject.call1(&JsValue::NULL, &err).unwrap();
            });
            request.set_onerror(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }
    })
}

fn request_to_promise(request: IdbRequest) -> Promise {
    Promise::new(&mut |resolve, reject| {
        {
            let req = request.clone();
            let resolve = resolve.clone();
            let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                let result = req.result().unwrap_or(JsValue::UNDEFINED);
                resolve.call1(&JsValue::NULL, &result).unwrap();
            });
            request.set_onsuccess(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        {
            let req = request.clone();
            let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                let err = req
                    .error()
                    .ok()
                    .flatten()
                    .map(Into::into)
                    .unwrap_or_else(|| JsValue::from_str("IDB request: unknown error"));
                reject.call1(&JsValue::NULL, &err).unwrap();
            });
            request.set_onerror(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }
    })
}

fn transaction_to_promise(tx: &IdbTransaction) -> Promise {
    Promise::new(&mut |resolve, reject| {
        let reject_err = reject.clone();
        let reject_abort = reject.clone();

        {
            let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            tx.set_oncomplete(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        {
            let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                reject_err
                    .call1(&JsValue::NULL, &JsValue::from_str("IDB transaction error"))
                    .unwrap();
            });
            tx.set_onerror(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        {
            let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                reject_abort
                    .call1(
                        &JsValue::NULL,
                        &JsValue::from_str("IDB transaction aborted"),
                    )
                    .unwrap();
            });
            tx.set_onabort(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }
    })
}

pub async fn open_db() -> Result<IdbDatabase, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No global `window` object"))?;

    let idb = window
        .indexed_db()?
        .ok_or_else(|| JsValue::from_str("IndexedDB is not available in this context"))?;

    let open_req = idb.open_with_u32(DB_NAME, DB_VERSION)?;
    let db_val = JsFuture::from(open_request_to_promise(open_req)).await?;

    db_val
        .dyn_into::<IdbDatabase>()
        .map_err(|_| JsValue::from_str("Unexpected: result is not an IdbDatabase"))
}

pub async fn load_database() -> Result<DatabaseState, JsValue> {
    info!("Loading database");
    let db = open_db().await?;
    let tx = db.transaction_with_str(STORE_NAME)?;
    let store = tx.object_store(STORE_NAME)?;
    let req = store.get(&JsValue::from_str(DB_KEY))?;

    let value = JsFuture::from(request_to_promise(req)).await?;

    info!("Loaded");

    if value.is_null() || value.is_undefined() {
        return Ok(DatabaseState::default());
    }
 
    let json = value.as_string().ok_or_else(|| {
        JsValue::from_str("Expected a JSON string in IndexedDB; got a different type")
    })?;

    let de = &mut serde_json::Deserializer::from_str(&json);
    let result: Result<DatabaseState, _> = serde_path_to_error::deserialize(de);
    result.map_err(|e| JsValue::from_str(&format!("Deserialization error with path {:#?}", e.path().to_string())))
}

pub async fn save_database(database: &DatabaseState) -> Result<(), JsValue> {
    let mut out = Vec::new();
    let ser = &mut serde_json::Serializer::new(&mut out);

    let mut track = serde_path_to_error::Track::new();
    let ps = serde_path_to_error::Serializer::new(ser, &mut track);

    database.serialize(ps).map_err(|e| JsValue::from_str(&format!("Serialization error: {:#?}\n Path: {:#?}", e, track.path().to_string())))?;
    let json = String::from_utf8(out).unwrap();

    let db = open_db().await?;
    let tx = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)?;
    let store = tx.object_store(STORE_NAME)?;
    store.put_with_key(&JsValue::from_str(&json), &JsValue::from_str(DB_KEY))?;

    JsFuture::from(transaction_to_promise(&tx)).await?;
    Ok(())
}
