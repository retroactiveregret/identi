use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use dioxus::{logger::tracing::info, prelude::*};
use js_sys::Reflect;
use palette::Srgb;
use serde::Deserialize;
use std::collections::HashSet;
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Blob, Request, RequestInit, Response};

use crate::models::*;

pub fn add_member(
    name: String,
    description: String,
    color: Option<Srgb<u8>>,
    avatar_asset_id: Option<Uuid>,
    banner_asset_id: Option<Uuid>,
) -> Result<Member, JsValue> {
    info!("Adding member");
    let db = use_context::<Signal<Database>>();
    let member = Member {
        id: Uuid::new_v4(),
        name,
        description,
        color,
        avatar_asset_id,
        banner_asset_id,
        archived: false,
        created_at: chrono::offset::Utc::now(),
    };
    db().members.write().insert(member.id, member.clone());
    Ok(member)
}

pub fn put_member(member: &Member) -> Result<Member, JsValue> {
    info!("Putting member");
    info!("{:#?}", member);
    let db = use_context::<Signal<Database>>();
    let member = member.to_owned();
    let mut db = db();
    let mut binding = db.members.write();
    let slot = binding
        .get_mut(&member.id)
        .ok_or_else(|| JsValue::from_str(&format!("Member {} not found", member.id)))?;
    *slot = member;
    Ok(slot.to_owned())
}

pub fn add_custom_field_values(values: Vec<CustomFieldValue>) -> Result<(), JsValue> {
    info!("Adding custom field values {:#?}", values);
    let db = use_context::<Signal<Database>>();
    db().custom_field_values.write().extend(values);
    Ok(())
}

pub fn end_current_period(ended_at: DateTime<Utc>) -> Result<Option<FrontPeriod>, JsValue> {
    info!("Ending current fronting period");
    let db = use_context::<Signal<Database>>();
    let mut db = db();
    let mut w = db.front_periods.write();
    let Some((_, fp)) = w.last_mut() else {
        return Ok(None);
    };
    if fp.ended_at.is_none() {
        fp.ended_at = Some(ended_at);
    }
    Ok(Some(fp.to_owned()))
}

pub fn add_period(
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    assignments: Vec<FrontPeriodAssignment>,
) -> Result<FrontPeriod, JsValue> {
    info!("Adding new fronting period");
    let db = use_context::<Signal<Database>>();
    let fp = FrontPeriod {
        id: Uuid::new_v4(),
        started_at,
        ended_at,
        assignments,
    };
    db().front_periods.write().insert(fp.id, fp.clone());
    Ok(fp)
}

pub fn switch(
    time: DateTime<Utc>,
    assignments: Vec<FrontPeriodAssignment>,
) -> Result<FrontPeriod, JsValue> {
    info!("Switching");
    let db = use_context::<Signal<Database>>();
    let mut binding = db();
    let mut write = binding.front_periods.write();
    if let Some((_, fp)) = write.last_mut() {
        let delta = (time - fp.started_at).num_seconds();
        if fp.ended_at.is_none() && delta >= 0 && delta < 20 {
            fp.assignments = assignments;
            return Ok(fp.to_owned());
        }

        if fp.ended_at.is_none() {
            fp.ended_at = Some(time);
        }
    }

    let fp = FrontPeriod {
        id: Uuid::new_v4(),
        started_at: time,
        ended_at: None,
        assignments,
    };
    write.insert(fp.id, fp.clone());
    Ok(fp)
}

pub fn put_front_period(
    id: Uuid,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    assignments: Vec<FrontPeriodAssignment>,
) -> Result<(), JsValue> {
    if started_at > ended_at {
        return Err(JsValue::from_str("End time must be after start time"));
    }
    let db = use_context::<Signal<Database>>();
    let mut binding = db();
    let idx = (binding.front_periods)().get_index_of(&id).unwrap();
    let mut write = binding.front_periods.write();

    {
        match write.get_index_mut(idx - 1) {
            Some((_, prev)) => {
                if prev.ended_at.unwrap() > started_at {
                    if started_at < prev.started_at {
                        return Err(JsValue::from_str(
                            "Invalid start time, please delete previous entry",
                        ));
                    }

                    let p = prev.clone();
                    *prev = FrontPeriod {
                        id: p.id,
                        started_at: p.started_at,
                        ended_at: Some(started_at),
                        assignments: p.assignments,
                    }
                }
            }
            None => {}
        }
    }

    {
        match write.get_index_mut(idx + 1) {
            Some((_, next)) => {
                if ended_at > next.started_at {
                    if ended_at > next.ended_at.unwrap() {
                        return Err(JsValue::from_str(
                            "Invalid end time, please delete proceeding entry",
                        ));
                    }

                    let n = next.clone();
                    *next = FrontPeriod {
                        id: n.id,
                        started_at: ended_at,
                        ended_at: n.ended_at,
                        assignments: n.assignments,
                    }
                }
            }
            None => {}
        }
    }

    {
        let fp = write.get_mut(&id).unwrap();
        *fp = FrontPeriod {
            id,
            started_at,
            ended_at: Some(ended_at),
            assignments,
        };
    }

    Ok(())
}

pub fn add_journal_entry(
    title: String,
    body: String,
    created_at: DateTime<Utc>,
    author_member_ids: Vec<Uuid>,
    content_warning: Option<String>,
) -> Result<JournalEntry, JsValue> {
    let db = use_context::<Signal<Database>>();
    let entry = JournalEntry {
        id: Uuid::new_v4(),
        title,
        body,
        created_at,
        updated_at: None,
        author_member_ids,
        content_warning,
    };
    db().journal_entries.write().insert(entry.id, entry.clone());
    Ok(entry)
}

pub fn put_journal_entry(
    id: Uuid,
    title: String,
    body: String,
    created_at: DateTime<Utc>,
    author_member_ids: Vec<Uuid>,
    content_warning: Option<String>,
) -> Result<JournalEntry, JsValue> {
    let db = use_context::<Signal<Database>>();
    let mut binding = db();
    let mut write = binding.journal_entries.write();
    let entry = write.get_mut(&id).unwrap();
    *entry = JournalEntry {
        id,
        title,
        body,
        created_at,
        updated_at: None,
        author_member_ids,
        content_warning,
    };
    Ok(entry.clone())
}

pub fn add_post(
    author_id: Option<Uuid>,
    mentions: HashSet<Uuid>,
    content: String,
    pinned: bool,
    created_at: DateTime<Utc>,
) -> Result<BoardPost, JsValue> {
    if content.is_empty() {
        return Err(JsValue::from_str("Post content must not be empty"));
    }

    let db = use_context::<Signal<Database>>();

    let post = BoardPost {
        id: Uuid::new_v4(),
        author_id,
        mentions,
        content,
        pinned,
        archived: false,
        created_at,
    };

    {
        let mut binding = db().board_posts;
        let mut board_posts = binding.write();

        if pinned {
            board_posts.insert(post.id, post.clone());
        } else {
            let insert_index = board_posts
                .iter()
                .rev()
                .take_while(|(_, p)| p.pinned)
                .count();

            if insert_index == 0 {
                board_posts.insert(post.id, post.clone());
            } else {
                let idx = board_posts.len() - insert_index;
                board_posts.shift_insert(idx, post.id, post.clone());
            }
        }
    }

    add_mentions(post.id, &post.mentions)?;
    Ok(post)
}

pub fn add_mentions(post_id: Uuid, mentioned_users: &HashSet<Uuid>) -> Result<(), JsValue> {
    let db = use_context::<Signal<Database>>();
    for user in mentioned_users {
        let id = Uuid::new_v4();
        db().user_mentions.write().insert(
            id,
            UserMention {
                id,
                user_id: *user,
                board_post_id: post_id,
                read: false,
            },
        );
    }
    Ok(())
}

pub fn archive_post(id: Uuid, archived: bool) -> Result<(), JsValue> {
    let db = use_context::<Signal<Database>>();
    let idx = (db().board_posts)()
        .get_index_of(&id)
        .ok_or_else(|| JsValue::from_str(&format!("Unable to find post {id}")))?;
    let mut db = db();
    let mut binding = db.board_posts.write();
    let post = binding
        .get_mut(&id)
        .ok_or_else(|| JsValue::from_str(&format!("Unable to find post {id}")))?;
    post.archived = archived;
    if post.pinned {
        binding.swap_indices(idx, 0);
    }
    Ok(())
}

pub fn mark_notification_read(id: Uuid, read: bool) -> Result<UserMention, JsValue> {
    let mut db = use_context::<Signal<Database>>()();
    let mut binding = db.user_mentions.write();
    let post = binding
        .get_mut(&id)
        .ok_or_else(|| JsValue::from_str(&format!("Unable to find mention {id}")))?;
    post.read = read;
    Ok(post.to_owned())
}

pub fn mark_all_notifications_read() -> Result<(), JsValue> {
    let db = use_context::<Signal<Database>>();
    for (_, mention) in db().user_mentions.write().iter_mut() {
        mention.read = true;
    }
    Ok(())
}

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
