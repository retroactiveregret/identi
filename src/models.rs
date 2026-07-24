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
