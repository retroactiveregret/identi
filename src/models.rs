use chrono::{self, DateTime, NaiveDate, Utc};
use dioxus::{logger::tracing::info, prelude::*};
use indexmap::IndexMap;
use palette::Srgb;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::{Deref, DerefMut},
};
use uuid::Uuid;

fn serialize_uuid_compat<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

fn deserialize_uuid_compat<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum UuidValue {
        String(String),
        Number(i64),
        U64(u64),
    }

    match UuidValue::deserialize(deserializer)? {
        UuidValue::String(value) => {
            if let Ok(uuid) = Uuid::parse_str(&value) {
                Ok(uuid)
            } else if let Ok(number) = value.parse::<u64>() {
                Ok(Uuid::from_u128(number as u128))
            } else {
                Err(serde::de::Error::custom(format!(
                    "invalid UUID value: {value}"
                )))
            }
        }
        UuidValue::Number(value) => Ok(Uuid::from_u128(value as u128)),
        UuidValue::U64(value) => Ok(Uuid::from_u128(value as u128)),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Database {
    pub members: Signal<IndexMap<Uuid, Member>>,
    pub taxonomy_terms: Signal<IndexMap<Uuid, TaxonomyTerm>>,
    pub taxonomy_assignments: Signal<IndexMap<Uuid, TaxonomyAssignment>>,
    pub custom_fields: Signal<IndexMap<Uuid, CustomField>>,
    pub custom_field_values: Signal<Vec<CustomFieldValue>>,
    pub front_periods: Signal<IndexMap<Uuid, FrontPeriod>>,
    pub journal_entries: Signal<IndexMap<Uuid, JournalEntry>>,
    pub board_posts: Signal<IndexMap<Uuid, BoardPost>>,
    pub user_mentions: Signal<IndexMap<Uuid, UserMention>>,
    pub settings: Signal<Settings>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DatabaseState {
    pub members: IndexMap<Uuid, Member>,
    pub taxonomy_terms: IndexMap<Uuid, TaxonomyTerm>,
    pub taxonomy_assignments: IndexMap<Uuid, TaxonomyAssignment>,
    pub custom_fields: IndexMap<Uuid, CustomField>,
    pub custom_field_values: Vec<CustomFieldValue>,
    pub front_periods: IndexMap<Uuid, FrontPeriod>,
    pub journal_entries: IndexMap<Uuid, JournalEntry>,
    pub board_posts: IndexMap<Uuid, BoardPost>,
    pub user_mentions: IndexMap<Uuid, UserMention>,
    pub settings: Settings,
}

impl From<Database> for DatabaseState {
    fn from(value: Database) -> Self {
        Self {
            members: (value.members)(),
            taxonomy_terms: (value.taxonomy_terms)(),
            taxonomy_assignments: (value.taxonomy_assignments)(),
            custom_fields: (value.custom_fields)(),
            custom_field_values: (value.custom_field_values)(),
            front_periods: (value.front_periods)(),
            journal_entries: (value.journal_entries)(),
            board_posts: (value.board_posts)(),
            user_mentions: (value.user_mentions)(),
            settings: (value.settings)(),
        }
    }
}

impl From<DatabaseState> for Database {
    fn from(value: DatabaseState) -> Self {
        Self {
            members: Signal::new(value.members),
            taxonomy_terms: Signal::new(value.taxonomy_terms),
            taxonomy_assignments: Signal::new(value.taxonomy_assignments),
            custom_fields: Signal::new(value.custom_fields),
            custom_field_values: Signal::new(value.custom_field_values),
            front_periods: Signal::new(value.front_periods),
            journal_entries: Signal::new(value.journal_entries),
            board_posts: Signal::new(value.board_posts),
            user_mentions: Signal::new(value.user_mentions),
            settings: Signal::new(value.settings),
        }
    }
}

impl Default for DatabaseState {
    fn default() -> Self {
        Self {
            members: Default::default(),
            taxonomy_terms: Default::default(),
            taxonomy_assignments: Default::default(),
            custom_fields: Default::default(),
            custom_field_values: Default::default(),
            front_periods: Default::default(),
            journal_entries: Default::default(),
            board_posts: Default::default(),
            user_mentions: Default::default(),
            settings: Default::default(),
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            members: Default::default(),
            taxonomy_terms: Default::default(),
            taxonomy_assignments: Default::default(),
            custom_fields: Default::default(),
            custom_field_values: Default::default(),
            front_periods: Default::default(),
            journal_entries: Default::default(),
            board_posts: Default::default(),
            user_mentions: Default::default(),
            settings: Default::default(),
        }
    }
}

impl Database {
    pub fn get_last_period(&self) -> Option<FrontPeriod> {
        self.front_periods.read().last().map(|(_, fp)| fp.clone())
    }

    pub fn get_active_period(&self) -> Option<FrontPeriod> {
        if let Some((_, fp)) = self.front_periods.read().last() {
            match &fp.ended_at {
                None => {
                    Some(fp.clone())
                }
                Some(_) => {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn find_custom_field_value(
        &self,
        field_id: Uuid,
        member_id: Uuid,
    ) -> Option<CustomFieldValue> {
        for value in (self.custom_field_values)() {
            if value.field_id == field_id && value.member_id == member_id {
                return Some(value.clone());
            }
        }
        info!("No value found for field {}", field_id);
        return None;
    }

    pub fn get_unarchived_board_posts(&self) -> Vec<BoardPost> {
        self.board_posts
            .read()
            .iter()
            .rev()
            .filter(|(_, p)| !p.archived)
            .map(|(_, p)| p.clone())
            .collect()
    }

    pub fn get_unarchived_board_posts_paginated(&self, n: usize, start: usize) -> Vec<BoardPost> {
        self.board_posts
            .read()
            .iter()
            .rev()
            .filter(|(_, p)| !p.archived)
            .skip(start)
            .take(n)
            .map(|(_, p)| p.clone())
            .collect()
    }

    pub fn add_member(
        &self,
        name: String,
        description: String,
        color: Option<Srgb<u8>>,
        avatar_asset_id: Option<Uuid>,
        banner_asset_id: Option<Uuid>,
    ) -> Result<Member, wasm_bindgen::JsValue> {
        info!("Adding member");
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
        let mut members = self.members;
        members.write().insert(member.id, member.clone());
        Ok(member)
    }

    pub fn put_member(&self, member: &Member) -> Result<Member, wasm_bindgen::JsValue> {
        info!("Putting member");
        info!("{:#?}", member);
        let member = member.to_owned();
        let mut members = self.members;
        let mut binding = members.write();
        let slot = binding
            .get_mut(&member.id)
            .ok_or_else(|| wasm_bindgen::JsValue::from_str(&format!("Member {} not found", member.id)))?;
        *slot = member;
        Ok(slot.to_owned())
    }

    pub fn add_custom_field_values(
        &self,
        values: Vec<CustomFieldValue>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        info!("Adding custom field values {:#?}", values);
        let mut custom_field_values = self.custom_field_values;
        custom_field_values.write().extend(values);
        Ok(())
    }

    pub fn end_current_period(
        &self,
        ended_at: DateTime<Utc>,
    ) -> Result<Option<FrontPeriod>, wasm_bindgen::JsValue> {
        info!("Ending current fronting period");
        let mut front_periods = self.front_periods;
        let mut w = front_periods.write();
        let Some((_, fp)) = w.last_mut() else {
            return Ok(None);
        };
        if fp.ended_at.is_none() {
            fp.ended_at = Some(ended_at);
        }
        Ok(Some(fp.to_owned()))
    }

    pub fn add_period(
        &self,
        started_at: DateTime<Utc>,
        ended_at: Option<DateTime<Utc>>,
        assignments: Vec<FrontPeriodAssignment>,
    ) -> Result<FrontPeriod, wasm_bindgen::JsValue> {
        info!("Adding new fronting period");
        let fp = FrontPeriod {
            id: Uuid::new_v4(),
            started_at,
            ended_at,
            assignments,
        };
        let mut front_periods = self.front_periods;
        front_periods.write().insert(fp.id, fp.clone());
        Ok(fp)
    }

    pub fn switch(
        &self,
        time: DateTime<Utc>,
        assignments: Vec<FrontPeriodAssignment>,
    ) -> Result<FrontPeriod, wasm_bindgen::JsValue> {
        info!("Switching");
        let mut front_periods = self.front_periods;
        let mut write = front_periods.write();
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
        &self,
        id: Uuid,
        started_at: DateTime<Utc>,
        ended_at: DateTime<Utc>,
        assignments: Vec<FrontPeriodAssignment>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        if started_at > ended_at {
            return Err(wasm_bindgen::JsValue::from_str("End time must be after start time"));
        }
        let mut front_periods = self.front_periods;
        let idx = front_periods.read().get_index_of(&id).unwrap();
        let mut write = front_periods.write();

        {
            match write.get_index_mut(idx - 1) {
                Some((_, prev)) => {
                    if prev.ended_at.unwrap() > started_at {
                        if started_at < prev.started_at {
                            return Err(wasm_bindgen::JsValue::from_str(
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
                            return Err(wasm_bindgen::JsValue::from_str(
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
        &self,
        title: String,
        body: String,
        created_at: DateTime<Utc>,
        author_member_ids: Vec<Uuid>,
        content_warning: Option<String>,
    ) -> Result<JournalEntry, wasm_bindgen::JsValue> {
        let entry = JournalEntry {
            id: Uuid::new_v4(),
            title,
            body,
            created_at,
            updated_at: None,
            author_member_ids,
            content_warning,
        };
        let mut journal_entries = self.journal_entries;
        journal_entries.write().insert(entry.id, entry.clone());
        Ok(entry)
    }

    pub fn put_journal_entry(
        &self,
        id: Uuid,
        title: String,
        body: String,
        created_at: DateTime<Utc>,
        author_member_ids: Vec<Uuid>,
        content_warning: Option<String>,
    ) -> Result<JournalEntry, wasm_bindgen::JsValue> {
        let mut journal_entries = self.journal_entries;
        let mut write = journal_entries.write();
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
        &self,
        author_id: Option<Uuid>,
        mentions: HashSet<Uuid>,
        content: String,
        pinned: bool,
        created_at: DateTime<Utc>,
    ) -> Result<BoardPost, wasm_bindgen::JsValue> {
        if content.is_empty() {
            return Err(wasm_bindgen::JsValue::from_str("Post content must not be empty"));
        }

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
            let mut board_posts_signal = self.board_posts;
            let mut board_posts = board_posts_signal.write();

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

        self.add_mentions(post.id, &post.mentions)?;
        Ok(post)
    }

    pub fn add_mentions(
        &self,
        post_id: Uuid,
        mentioned_users: &HashSet<Uuid>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let mut user_mentions = self.user_mentions;
        for user in mentioned_users {
            let id = Uuid::new_v4();
            user_mentions.write().insert(
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

    pub fn archive_post(&self, id: Uuid, archived: bool) -> Result<(), wasm_bindgen::JsValue> {
        let mut board_posts = self.board_posts;
        let idx = board_posts.read().get_index_of(&id).ok_or_else(|| {
            wasm_bindgen::JsValue::from_str(&format!("Unable to find post {id}"))
        })?;
        let mut binding = board_posts.write();
        let post = binding
            .get_mut(&id)
            .ok_or_else(|| wasm_bindgen::JsValue::from_str(&format!("Unable to find post {id}")))?;
        post.archived = archived;
        if post.pinned {
            binding.swap_indices(idx, 0);
        }
        Ok(())
    }

    pub fn mark_notification_read(
        &self,
        id: Uuid,
        read: bool,
    ) -> Result<UserMention, wasm_bindgen::JsValue> {
        let mut user_mentions = self.user_mentions;
        let mut binding = user_mentions.write();
        let post = binding
            .get_mut(&id)
            .ok_or_else(|| wasm_bindgen::JsValue::from_str(&format!("Unable to find mention {id}")))?;
        post.read = read;
        Ok(post.to_owned())
    }

    pub fn mark_all_notifications_read(&self) -> Result<(), wasm_bindgen::JsValue> {
        let mut user_mentions = self.user_mentions;
        for (_, mention) in user_mentions.write().iter_mut() {
            mention.read = true;
        }
        Ok(())
    }
}

fn default_created_at() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Member {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub color: Option<Srgb<u8>>,
    #[serde(default)]
    pub avatar_asset_id: Option<Uuid>,
    #[serde(default)]
    pub banner_asset_id: Option<Uuid>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaxonomyTerm {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Srgb<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaxonomyAssignment {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub term_id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub subject_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomField {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub id: Uuid,
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    Text,
    Number,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomFieldValue {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub field_id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub member_id: Uuid,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Text(String),
    Number(i64),
    Boolean(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrontPeriod {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub assignments: Vec<FrontPeriodAssignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrontPeriodAssignment {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub member_id: Uuid,
    #[serde(default)]
    pub front_role: FrontRole,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub note: String,
}

fn default_confidence() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum FrontRole {
    Primary,
    CoFront,
    CoCon,
    Influencing,
    Custom(String),

    #[default]
    Unknown
}

impl ToString for FrontRole {
    fn to_string(&self) -> String {
        match self {
            FrontRole::Primary => "primary",
            FrontRole::CoFront => "cofront",
            FrontRole::CoCon => "cocon",
            FrontRole::Influencing => "influencing",
            FrontRole::Custom(s) => s,
            FrontRole::Unknown => "unknown",
        }.into()
    }
}

impl From<String> for FrontRole {
    fn from(string: String) -> Self {
        let s = string.as_str();
        match s {
            "primary" => Self::Primary,
            "cofront" => Self::CoFront, 
            "cocon" => Self::CoCon,
            "influencing" => Self::Influencing,
            "unknown" => Self::Unknown,
            s => Self::Custom(s.into())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JournalEntry {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub author_member_ids: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardPost {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_id: Option<Uuid>,
    #[serde(default)]
    pub mentions: HashSet<Uuid>,
    pub content: String,
    pub pinned: bool,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMention {
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub user_id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid_compat",
        deserialize_with = "deserialize_uuid_compat"
    )]
    pub board_post_id: Uuid,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub theme: String,
    pub blur_banners: bool,
    pub outline_notifications: bool,
    pub notification_popup: bool,
    pub notification_banner: bool,
    pub front_history_show: usize,
    pub board_show: usize,
    pub twelve_hour: bool,
    pub banner_opacity: usize,
    pub overlay_neutral: bool,

    pub sanitize_html: bool,
    pub app_lock: Option<String>,
    pub dev_tools: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dracula".into(),
            blur_banners: true,
            outline_notifications: false,
            notification_popup: false,
            notification_banner: true,
            front_history_show: 10,
            board_show: 10,
            twelve_hour: true,
            banner_opacity: 30,
            overlay_neutral: true,

            sanitize_html: true,
            app_lock: None,
            dev_tools: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status {
    pub message: String,
    pub level: StatusLevel,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StatusLevel {
    Success,
    Warning,
    Error,
}

impl Status {
    pub fn set_message<T>(&mut self, msg: T, level: StatusLevel)
    where
        T: ToString,
    {
        info!("{}", msg.to_string());
        self.message = msg.to_string();
        self.time = chrono::Utc::now();
        self.level = level;
    }

    pub fn set_level(&mut self, level: StatusLevel) {
        self.level = level;
    }

    pub fn display_time_check(&self) -> bool {
        return (chrono::Utc::now() - self.time).num_seconds() < 5;
    }

    pub fn alert_class(&self) -> String {
        match self.level {
            StatusLevel::Success => "alert-success alert-soft".into(),
            StatusLevel::Warning => "alert-warning".into(),
            StatusLevel::Error => "alert-error".into(),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)?;
        Ok(())
    }
}

impl Default for Status {
    fn default() -> Self {
        Self {
            message: Default::default(),
            level: StatusLevel::Success,
            time: DateTime::<Utc>::UNIX_EPOCH,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMentionsLookup(pub HashMap<Uuid, Vec<Uuid>>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JournalDates(pub HashMap<NaiveDate, Vec<Uuid>>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwitchDates(pub HashMap<NaiveDate, Vec<Uuid>>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostDates(pub HashMap<NaiveDate, Vec<Uuid>>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomFieldValueLookup(pub HashMap<(Uuid, Uuid), CustomFieldValue>);

impl Deref for UserMentionsLookup {
    type Target = HashMap<Uuid, Vec<Uuid>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UserMentionsLookup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for CustomFieldValueLookup {
    type Target = HashMap<(Uuid, Uuid), CustomFieldValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CustomFieldValueLookup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for JournalDates {
    type Target = HashMap<NaiveDate, Vec<Uuid>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JournalDates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for SwitchDates {
    type Target = HashMap<NaiveDate, Vec<Uuid>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SwitchDates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for PostDates {
    type Target = HashMap<NaiveDate, Vec<Uuid>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PostDates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
