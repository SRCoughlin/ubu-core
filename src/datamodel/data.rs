#![allow(deprecated)]
#![allow(dead_code)]
#![allow(unreachable_code)]

// use std::vec;
use chrono::{Utc, DateTime};
use std::fmt;
use std::fmt::{Display,Formatter};
use rusqlite::types::*;
use serde::{Deserialize, Serialize,Serializer,Deserializer};
use serde::ser::{/*Serialize, Serializer,*/ SerializeStruct};
use serde::de::{self, Visitor};
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use crate::datamodel::database::*;

#[derive(Debug, Copy, Clone)]
pub struct Duration {
    d: i64,
}
impl Into<Duration> for chrono::Duration {
    fn into(self) -> Duration {
        Duration { d: self.num_seconds() }
    }
}
impl From<&chrono::Duration> for Duration {
    fn from(item: &chrono::Duration) -> Self {
        Duration { d: item.num_seconds() }
    }
}
impl ToSql for Duration {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.d.into())
    }
}
impl FromSql for Duration {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                Ok(Duration { d: vi64.try_into().unwrap() })
            },
            _ => panic!("Unknown Duration value: {:?}", value),
        }
    }
}
impl Duration {
    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.d.to_be_bytes()
    }
    pub fn to_chrono_duration(self) -> chrono::Duration {
        chrono::Duration::seconds(self.d)
    }
}
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.d)
    }
}
struct DurationVisitor;
impl<'de> Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^61 and 2^61")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Duration { d: value })
    }
}
impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(DurationVisitor)
    }
}

#[derive(Debug)]
pub enum DataError {
    RusqliteError(rusqlite::Error),
    CircularDependency,
    InconsistentPlan,
}
impl From<rusqlite::Error> for DataError {
    fn from(error: rusqlite::Error) -> Self {
        DataError::RusqliteError(error)
    }
}

//Notification
#[derive(Debug, Serialize, Copy, Clone)]
pub enum NotificationType {
    Email,
    GitHub,
}

#[derive(Debug, Serialize, Copy, Clone)]
pub enum NotificationBodyType {
    HTML,
}
impl ToSql for NotificationBodyType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            NotificationBodyType::HTML => Ok(1.into()),
        }
    }
}
impl FromSql for NotificationBodyType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                match vi64 {
                    1 => Ok(NotificationBodyType::HTML.into()),
                    _ => panic!("Unknown NotificationBodyType value: {}", vi64),
                }
            },
            _ => panic!("Unknown NotificationBodyType value: {:?}", value),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct EmailNotification {
    pub date: DateTime<Utc>,
    pub message_id: String,
    pub from: String,
    pub subject: String,
    pub body_type: NotificationBodyType,
    pub body: String,
    pub text: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct GitHubNotification {
    pub date: DateTime<Utc>,
    pub repo: String,
    pub thread: String,
    pub from: String,
    pub body: String,
    pub label: String,
}

//Event
#[derive(Debug, Copy, Clone)]
pub enum EventColor {
    Uncolored,
}
impl ToSql for EventColor {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            EventColor::Uncolored => Ok(1.into()),
        }
    }
}
impl FromSql for EventColor {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                match vi64 {
                    1 => Ok(EventColor::Uncolored.into()),
                    _ => panic!("Unknown EventColor value: {}", vi64),
                }
            },
            _ => panic!("Unknown EventColor value: {:?}", value),
        }
    }
}

//MacroTask
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum MacroTaskType {
    Email,
    GitHub,
}
impl Display for MacroTaskType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            MacroTaskType::Email => write!(f, "Email"),
            MacroTaskType::GitHub => write!(f, "GitHub"),
        }
    }
}
impl From<&MacroTaskType> for u8 {
    fn from(c: &MacroTaskType) -> Self {
        match c {
            MacroTaskType::Email => 1.into(),
            MacroTaskType::GitHub => 2.into(),
        }
    }
}
impl ToSql for MacroTaskType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            MacroTaskType::Email => Ok(1.into()),
            MacroTaskType::GitHub => Ok(2.into()),
        }
    }
}
impl FromSql for MacroTaskType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                match vi64 {
                    1 => Ok(MacroTaskType::Email.into()),
                    2 => Ok(MacroTaskType::GitHub.into()),
                    _ => panic!("Unknown MacroTaskType value: {}", vi64),
                }
            },
            _ => panic!("Unknown MacroTaskType value: {:?}", value),
        }
    }
}

/// Task ///
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum TaskType {
    Idea,
    Action,
    Super,
    Macro,
    Micro,
    Fixed,
}
impl From<&TaskType> for u8 {
    fn from(c: &TaskType) -> Self {
        match c {
            TaskType::Idea => 1.into(),
            TaskType::Action => 2.into(),
            TaskType::Super => 3.into(),
            TaskType::Macro => 4.into(),
            TaskType::Micro => 5.into(),
            TaskType::Fixed => 6.into(),
        }
    }
}
impl ToSql for TaskType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            TaskType::Idea => Ok(1.into()),
            TaskType::Action => Ok(2.into()),
            TaskType::Super => Ok(3.into()),
            TaskType::Macro => Ok(4.into()),
            TaskType::Micro => Ok(5.into()),
            TaskType::Fixed => Ok(6.into()),
        }
    }
}
impl FromSql for TaskType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                match vi64 {
                    1 => Ok(TaskType::Idea.into()),
                    2 => Ok(TaskType::Action.into()),
                    3 => Ok(TaskType::Super.into()),
                    4 => Ok(TaskType::Macro.into()),
                    5 => Ok(TaskType::Micro.into()),
                    6 => Ok(TaskType::Fixed.into()),
                    _ => panic!("Unknown TaskType value: {}", vi64),
                }
            },
            _ => panic!("Unknown TaskType value: {:?}", value),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Status {
    Pending,
    Active,
    Complete,
}
impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "Pending"),
            Status::Active => write!(f, "Active"),
            Status::Complete => write!(f, "Complete"),
        }
    }
}
impl From<&Status> for u8 {
    fn from(c: &Status) -> Self {
        match c {
            Status::Pending => 1.into(),
            Status::Active => 2.into(),
            Status::Complete => 3.into(),
        }
    }
}
impl ToSql for Status {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            Status::Pending => Ok(1.into()),
            Status::Active => Ok(2.into()),
            Status::Complete => Ok(3.into()),
        }
    }
}
impl FromSql for Status {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                match vi64 {
                    1 => Ok(Status::Pending.into()),
                    2 => Ok(Status::Active.into()),
                    3 => Ok(Status::Complete.into()),
                    _ => panic!("Unknown Status value: {}", vi64),
                }
            },
            _ => panic!("Unknown Status value: {:?}", value),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ID {
    id: [u8;32],
}
impl Into<ID> for [u8;32] {
    fn into(self) -> ID {
        ID { id: self }
    }
}
impl Into<ID> for &[u8] {
    fn into(self) -> ID {
        let mut id: [u8; 32] = Default::default();
        id.copy_from_slice(self);
        ID { id: id }
    }
}
impl From<&[u8;32]> for ID {
    fn from(item: &[u8;32]) -> Self {
        ID { id: item.clone() }
    }
}
impl ToSql for ID {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.id.to_vec().into())
    }
}
impl FromSql for ID {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Blob(b) => {
                Ok(ID { id: b[0..32].try_into().unwrap() })
            },
            _ => panic!("Unknown ID value: {:?}", value),
        }
    }
}
impl ID {
    fn default() -> Self {
        ID {
            id: <[u8;32]>::default(),
        }
    }
    pub fn as_bytes(&self) -> &[u8;32] {
        &self.id
    }
}

//Deterministic ID
pub type TaskID = ID;
pub fn create_task_id(
    parent_id: &Option<TaskID>,
    created: &DateTime<Utc>,
    name: &String,
    task_type: &TaskType,
    status: &Status,
    macro_task_type: &Option<MacroTaskType>,
    start: &Option<DateTime<Utc>>,
    due: &Option<DateTime<Utc>>,
    duration: &Option<Duration>,
    body: &Option<String>,
    meta1: &Option<String>,
    meta2: &Option<String>,
    meta3: &Option<String>,
    value: &Option<f64>,
) -> TaskID {
    let mut hasher = Sha256::new();
    match parent_id {
        Some(parent_id) => {
            hasher.update(parent_id.as_bytes());
        }
        None => {
            hasher.update(TaskID::default().as_bytes());
        }
    }
    hasher.update(created.timestamp().to_be_bytes());
    {
        let mut sub_hasher = Sha256::new();
        sub_hasher.update(name.as_bytes());
        let sub_hash: [u8;32] = sub_hasher.finalize().into();

        hasher.update(sub_hash);
    }
    hasher.update((u8::from(task_type)).to_be_bytes());
    hasher.update((u8::from(status)).to_be_bytes());
    match macro_task_type {
        Some(macro_task_type) => {
            hasher.update((u8::from(macro_task_type)).to_be_bytes());
        }
        None => {
            hasher.update(0u8.to_be_bytes());
        }
    }
    match start {
        Some(start) => {
            hasher.update(start.timestamp().to_be_bytes());
        }
        None => {
            hasher.update(0i64.to_be_bytes());
        }
    }
    match due {
        Some(due) => {
            hasher.update(due.timestamp().to_be_bytes());
        }
        None => {
            hasher.update(0i64.to_be_bytes());
        }
    }
    match duration {
        Some(duration) => {
            hasher.update(duration.to_be_bytes());
        }
        None => {
            hasher.update(0u64.to_be_bytes());
        }
    }
    match body {
        Some(body) => {
            let mut sub_hasher = Sha256::new();
            sub_hasher.update(body.as_bytes());
            let sub_hash: [u8;32] = sub_hasher.finalize().into();

            hasher.update(sub_hash);
        }
        None => {
            hasher.update(<[u8;32]>::default());
        }
    }
    match meta1 {
        Some(meta1) => {
            let mut sub_hasher = Sha256::new();
            sub_hasher.update(meta1.as_bytes());
            let sub_hash: [u8;32] = sub_hasher.finalize().into();

            hasher.update(sub_hash);
        }
        None => {
            hasher.update(<[u8;32]>::default());
        }
    }
    match meta2 {
        Some(meta2) => {
            let mut sub_hasher = Sha256::new();
            sub_hasher.update(meta2.as_bytes());
            let sub_hash: [u8;32] = sub_hasher.finalize().into();

            hasher.update(sub_hash);
        }
        None => {
            hasher.update(<[u8;32]>::default());
        }
    }
    match meta3 {
        Some(meta3) => {
            let mut sub_hasher = Sha256::new();
            sub_hasher.update(meta3.as_bytes());
            let sub_hash: [u8;32] = sub_hasher.finalize().into();

            hasher.update(sub_hash);
        }
        None => {
            hasher.update(<[u8;32]>::default());
        }
    }
    match value {
        Some(value) => {
            let mut sub_hasher = Sha256::new();
            sub_hasher.update(value.to_be_bytes());
            let sub_hash: [u8;32] = sub_hasher.finalize().into();

            hasher.update(sub_hash);
        }
        None => {
            hasher.update(<[u8;32]>::default());
        }
    }
    let hash: [u8;32] = hasher.finalize().into();
    hash.into()
}

pub trait TaskTrait {
    //Straight Properties
    fn task_id(&self)-> TaskID;
    fn parent(&self)-> Option<TaskID>;
    fn created(&self) -> DateTime<Utc>;
    fn name(&self) -> String;
    fn task_type(&self) -> TaskType;
    fn macro_task_type(&self) -> Option<MacroTaskType>;
    fn status(&self) -> Status;
    fn start(&self) -> Option<DateTime<Utc>>;
    fn due(&self) -> Option<DateTime<Utc>>;
    fn duration(&self) -> Option<Duration>;
    fn body(&self) -> Option<String>;
    fn meta1(&self) -> Option<String>;
    fn meta2(&self) -> Option<String>;
    fn meta3(&self) -> Option<String>;
    fn value(&self) -> Option<f64>;

    //Derived Properties
    fn dependencies(&self) -> Option<Vec<TaskID>>;
    fn children(&self) -> Option<Vec<TaskID>>;

    //Serialize
    fn serialize(&self, children: Vec<Task>) -> Task where Self: Sized;
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub task_id: TaskID,
    pub parent_id: Option<TaskID>,
    created: DateTime<Utc>,
    name: String,
    task_type: TaskType,
    macro_task_type: Option<MacroTaskType>,
    status: Status,
    start: Option<DateTime<Utc>>,
    due: Option<DateTime<Utc>>,
    duration: Option<Duration>,
    body: Option<String>,
    meta1: Option<String>,
    meta2: Option<String>,
    meta3: Option<String>,
    value: Option<f64>,

    dependencies: Option<Vec<TaskID>>,
    children: Option<Vec<Task>>,
}
impl Task {
    pub fn new(
        parent_id: Option<TaskID>,
        created: DateTime<Utc>,
        name: String,
        task_type: TaskType,
        status: Status,
        macro_task_type: Option<MacroTaskType>,
        start: Option<DateTime<Utc>>,
        due: Option<DateTime<Utc>>,
        duration: Option<Duration>,
        body: Option<String>,
        meta1: Option<String>,
        meta2: Option<String>,
        meta3: Option<String>,
        value: Option<f64>,
    ) -> Result<Self,DataError> {
        let task_id = create_task_id(
            &parent_id,
            &created,
            &name,
            &task_type,
            &status,
            &macro_task_type,
            &start,
            &due,
            &duration,
            &body,
            &meta1,
            &meta2,
            &meta3,
            &value,
        );
        let parent_id = if let Some(parent_id) = parent_id {
            Some(parent_id.clone())
        } else {
            None
        };
        let task = Task {
            task_id: task_id,
            parent_id: parent_id,
            created: created,
            name: name,
            task_type: task_type,
            status: status,
            macro_task_type: macro_task_type,
            start: start,
            due: due,
            duration: duration,
            body: body,
            meta1: meta1,
            meta2: meta2,
            meta3: meta3,
            value: value,
            dependencies: None,
            children: None,
        };
        Ok(task)
    }
    pub fn serialize<'a>(t: &dyn TaskTrait, children: Option<Vec<Task>>) -> Result<Self,DataError> {
        let parent_id = if let Some(parent_id) = t.parent() {
            Some(parent_id.clone())
        } else {
            None
        };
        Ok(Task {
            task_id: t.task_id().clone(),
            parent_id: parent_id,
            created: t.created(),
            name: t.name(),
            task_type: t.task_type(),
            status: t.status(),
            macro_task_type: t.macro_task_type(),
            start: t.start(),
            due: t.due(),
            duration: t.duration(),
            body: if let Some(s) = t.body() { Some(s.clone()) } else { None },
            meta1: if let Some(s) = t.meta1() { Some(s.clone()) } else { None },
            meta2: if let Some(s) = t.meta2() { Some(s.clone()) } else { None },
            meta3: if let Some(s) = t.meta3() { Some(s.clone()) } else { None },
            value: t.value(),
            dependencies: t.dependencies(),
            children: children,
        })
    }
}

//Dependency
#[derive(Debug, Serialize, Copy, Clone, PartialEq)]
pub enum DependencyStatus {
    Active,
    Complete,
}
impl From<&DependencyStatus> for u8 {
    fn from(c: &DependencyStatus) -> Self {
        match c {
            DependencyStatus::Active => 1.into(),
            DependencyStatus::Complete => 2.into(),
        }
    }
}
impl ToSql for DependencyStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            DependencyStatus::Active => Ok(1.into()),
            DependencyStatus::Complete => Ok(2.into()),
        }
    }
}
impl FromSql for DependencyStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                match vi64 {
                    1 => Ok(DependencyStatus::Active.into()),
                    2 => Ok(DependencyStatus::Complete.into()),
                    _ => panic!("Unknown DependencyStatus value: {}", vi64),
                }
            },
            _ => panic!("Unknown DependencyStatus value: {:?}", value),
        }
    }
}

/// Calendar ///

//Event
pub type EventID = ID;

pub trait Event {
    fn id(&self) -> EventID;
    fn calendar(&self)-> CalendarID;
    fn task(&self)-> TaskID;
    fn from(&self) -> DateTime<Utc>;
    fn until(&self) -> DateTime<Utc>;
}
impl Serialize for dyn Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Event", 5)?;//the number of fields in the struct
        state.serialize_field("id", &self.id())?;
        state.serialize_field("calendar", &self.calendar())?;
        state.serialize_field("task", &self.task())?;
        state.serialize_field("from", &self.from())?;
        state.serialize_field("until", &self.until())?;
        state.end()
    }
}

//Algorithm
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Algorithm {
    Trivial,
    Direct,
}
impl ToSql for Algorithm {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            Algorithm::Trivial => Ok(1.into()),
            Algorithm::Direct => Ok(2.into()),
        }
    }
}
impl FromSql for Algorithm {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                match vi64 {
                    1 => Ok(Algorithm::Trivial.into()),
                    2 => Ok(Algorithm::Direct.into()),
                    _ => panic!("Unknown Algorithm value: {}", vi64),
                }
            },
            _ => panic!("Unknown Algorithm value: {:?}", value),
        }
    }
}
impl From<&Algorithm> for u8 {
    fn from(c: &Algorithm) -> Self {
        match c {
            Algorithm::Trivial => 1.into(),
            Algorithm::Direct => 2.into(),
        }
    }
}

//Calendar
//Deterministic ID
pub type CalendarID = ID;
pub fn create_calendar_id(
    parent_id: &Option<CalendarID>,
    algorithm: &Algorithm,
    from: &DateTime<Utc>,
    until: &DateTime<Utc>,
) -> CalendarID {
    let mut hasher = Sha256::new();
    match parent_id {
        Some(parent_id) => {
            hasher.update(parent_id.as_bytes());
        }
        None => {
            hasher.update(TaskID::default().as_bytes());
        }
    }
    hasher.update((u8::from(algorithm)).to_be_bytes());
    hasher.update(from.timestamp().to_be_bytes());
    hasher.update(until.timestamp().to_be_bytes());
    let hash: [u8;32] = hasher.finalize().into();
    hash.into()
}

pub trait CalendarTrait {
    //Straight Properties
    fn calendar_id(&self) -> CalendarID;
    fn parent(&self) -> Option<CalendarID>;
    fn algorithm(&self) -> Algorithm;
    fn from(&self) -> DateTime<Utc>;
    fn until(&self) -> DateTime<Utc>;

    //Derived Properties
    fn value(&self) -> Option<f64>;
    fn current_event(&self)-> Option<EventID>;
    fn current_task(&self)-> Option<TaskID>;
    fn events(&self) -> Vec<EventID>;

    //Virtual Properties
    fn average_value(&self)-> Option<f64> {
        if let Some(value) = self.value() {
            let duration = self.from() - self.until();
            let denominator = duration.num_seconds() as f64;
            Some(value / denominator)
        } else {
            None
        }
    }

    //Serialize
    fn serialize(&self) -> Calendar where Self: Sized;
}
// impl Serialize for dyn CalendarTrait {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut state = serializer.serialize_struct("Calendar", 7)?;//the number of fields in the struct
//         state.serialize_field("id", &self.id())?;
//         state.serialize_field("parent", &self.parent())?;
//         state.serialize_field("algorithm", &self.algorithm())?;
//         state.serialize_field("from", &self.from())?;
//         state.serialize_field("until", &self.until())?;
//         state.serialize_field("value", &self.value())?;
//         state.serialize_field("events", &self.events())?;
//         state.end()
//     }
// }
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub calendar_id: CalendarID,
    pub parent_id: Option<CalendarID>,
    pub algorithm: Algorithm,
    pub from: DateTime<Utc>,
    pub until: DateTime<Utc>,

    pub value: Option<f64>,
}
impl Calendar {
    pub fn new(
        parent_id: Option<CalendarID>,
        algorithm: Algorithm,
        from: DateTime<Utc>,
        until: DateTime<Utc>,
        value: Option<f64>,
    ) -> Result<Self,DataError> {
        let calendar_id = create_calendar_id(
            &parent_id,
            &algorithm,
            &from,
            &until,
        );
        let parent_id = if let Some(parent_id) = parent_id {
            Some(parent_id.clone())
        } else {
            None
        };
        let calendar = Calendar {
            calendar_id: calendar_id,
            parent_id: parent_id,
            algorithm: algorithm,
            from: from,
            until: until,
            value: value,
        };
        Ok(calendar)
    }
    pub fn serialize<'a>(t: &dyn CalendarTrait) -> Result<Self,DataError> {
        let parent_id = if let Some(parent_id) = t.parent() {
            Some(parent_id.clone())
        } else {
            None
        };
        Ok(Calendar {
            calendar_id: t.calendar_id().clone(),
            parent_id: parent_id,
            algorithm: t.algorithm(),
            from: t.from(),
            until: t.until(),
            value: t.value(),
        })
    }
}    

/// Planner ///
pub trait Planner {
    //Generic functions
    fn now(&self) -> Result<DateTime<Utc>,DataError>;

    //Global functions
    fn detect_dependency_cycles(&self) -> Result<bool,DataError>;

    //Get functions
    fn get_task<T>(&self, id: TaskID) -> Result<T,DataError> where T: TaskTrait;
    fn get_event<T>(&self, id: EventID) -> Result<T,DataError> where T: Event;
    fn get_calendar(&self, id: CalendarID) -> Result<Calendar,DataError>;

    //Task functions
    fn active_tasks(&self, limit: u32) -> Vec<Task>;
    fn open_tasks(&self, limit: u32) -> Vec<Task>;
    fn available_tasks(&self, limit: u32) -> Vec<Task>;
    fn set_dependency(&self, before: &TaskID, after: &TaskID) -> Result<(),DataError>;
    fn unset_dependency(&self, before: &TaskID, after: &TaskID) -> Result<(),DataError>;

    //Calendar functions
    fn create_calendar(&self, algorithm: Algorithm, from: DateTime<Utc>, until: DateTime<Utc>) -> Result<Calendar,DataError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TaskTraitDummy {
        task: Task,
    }
    impl TaskTraitDummy {
        fn new() -> Self {
            TaskTraitDummy {
                task: Task::new(
                    None,
                    chrono::offset::Utc::now(),
                    "".to_string(),
                    TaskType::Idea,
                    Status::Active,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ).unwrap(),
            }
        }
    }
    impl TaskTrait for TaskTraitDummy {
        //Straight Properties
        fn task_id(&self) -> TaskID {
            self.task.task_id.clone()
        }
        fn parent(&self) -> Option<TaskID> {
            self.task.parent_id.clone()
        }
        fn created(&self) -> DateTime<Utc> {
            self.task.created
        }
        fn name(&self) -> String {
            self.task.name.clone()
        }
        fn task_type(&self) -> TaskType {
            self.task.task_type
        }
        fn macro_task_type(&self) -> Option<MacroTaskType> {
            self.task.macro_task_type
        }
        fn status(&self) -> Status {
            self.task.status
        }
        fn start(&self) -> Option<DateTime<Utc>> {
            self.task.start
        }
        fn due(&self) -> Option<DateTime<Utc>> {
            self.task.due
        }
        fn duration(&self) -> Option<Duration> {
            self.task.duration
        }
        fn body(&self) -> Option<String> {
            if let Some(s) = &self.task.body { Some(s.clone()) } else { None }
        }
        fn meta1(&self) -> Option<String> {
            if let Some(s) = &self.task.meta1 { Some(s.clone()) } else { None }
        }
        fn meta2(&self) -> Option<String> {
            if let Some(s) = &self.task.meta2 { Some(s.clone()) } else { None }
        }
        fn meta3(&self) -> Option<String> {
            if let Some(s) = &self.task.meta3 { Some(s.clone()) } else { None }
        }
        fn value(&self) -> Option<f64> {
            self.task.value
        }

        //Derived Properties
        fn dependencies(&self) -> Option<Vec<TaskID>> {
            None
        }
        fn children(&self) -> Option<Vec<TaskID>> {
            None
        }

        fn serialize(&self, _children: Vec<Task>)  -> Task {
            todo!();
        }
    }

    #[test]
    fn task_new() {
        let _ = TaskTraitDummy::new();
    }

    #[test]
    fn task_serialize() {
        let t = TaskTraitDummy::new();

        let _ = Task::serialize(&t, None).unwrap();
    }
}