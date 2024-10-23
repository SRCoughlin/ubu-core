#![allow(deprecated)]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused)]

use rusqlite::{params, Connection, Result};
use rusqlite::types::*;
use chrono::{Utc, DateTime};
use chrono::naive::{Days};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::marker::PhantomData;
use std::collections::HashMap;
use std::convert::TryInto;
use std::cmp;
use crate::datamodel::data::*;

// Database
struct Sqlite;
type PrimaryKey = i64;

// #[derive(Debug, Clone)]
pub struct Database {
    conn: Connection,
}

#[derive(Debug, Copy, Clone)]
pub enum RowProcess {
    Leave,
    Delete,
}

// pub struct DatabaseEvent<'a> {
//     db: &'a Database,
//     id: PrimaryKey,
//     task_id: PrimaryKey,
// }
// impl <'a> DatabaseEvent<'a> {
//     //Straight Properties
//     pub fn created(&self) -> DateTime<Utc> {
//         todo!();
//     }
//     pub fn from(&self) -> DateTime<Utc> {
//         todo!();
//     }
//     pub fn to(&self) -> DateTime<Utc> {
//         todo!();
//     }
//     pub fn description(&self) -> Option<String> {
//         todo!();
//     }

//     //Derived Properties
//     pub fn all_day(&self) -> bool {
//         todo!();
//     }

//     //Methods
// }

// #[derive(Serialize, Clone)]
// #[serde(rename_all = "camelCase")]
pub struct DatabaseTask<'a> {
    // #[serde(skip_serializing)]
    db: &'a Database,
    id: PrimaryKey,
}
impl PartialEq for DatabaseTask<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl <'a> TaskTrait for DatabaseTask<'a> {
    //Straight Properties
    fn task_id(&self) -> TaskID {
        self.db.conn.query_row("SELECT taskid FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn parent(&self) -> Option<TaskID> {
        match self.db.conn.query_row("SELECT parent.taskid FROM task child INNER JOIN task parent ON child.parent=parent.id WHERE child.id=?", [self.id], |r| r.get(0)) {
            Ok(task_id) => task_id,
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            _ => panic!(),
        }
    }
    fn created(&self) -> DateTime<Utc> {
        self.db.conn.query_row("SELECT created FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn name(&self) -> String {
        self.db.conn.query_row("SELECT name FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn task_type(&self) -> TaskType {
        self.db.conn.query_row("SELECT tasktype FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn macro_task_type(&self) -> Option<MacroTaskType> {
        self.db.conn.query_row("SELECT macrotasktype FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn status(&self) -> Status {
        self.db.conn.query_row("SELECT status FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn start(&self) -> Option<DateTime<Utc>> {
        self.db.conn.query_row("SELECT start FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn due(&self) -> Option<DateTime<Utc>> {
        self.db.conn.query_row("SELECT due FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn duration(&self) -> Option<Duration> {
        self.db.conn.query_row("SELECT duration FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn body(&self) -> Option<String> {
        self.db.conn.query_row("SELECT body FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn meta1(&self) -> Option<String> {
        self.db.conn.query_row("SELECT meta1 FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn meta2(&self) -> Option<String> {
        self.db.conn.query_row("SELECT meta2 FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn meta3(&self) -> Option<String> {
        self.db.conn.query_row("SELECT meta3 FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn value(&self) -> Option<f64> {
        self.db.conn.query_row("SELECT value FROM task WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }

    //Derived Properties
    fn dependencies(&self) -> Option<Vec<TaskID>> {
        let query = "SELECT task.taskid FROM task INNER JOIN dependency ON dependency.before=task.id WHERE dependency.after=? AND dependency.status=1";
        let mut stmt = self.db.conn.prepare(query).unwrap();
        let task_id_iter = stmt.query_map([self.id], |row| {
                Ok(row.get(0)?)
            }).unwrap();

        let mut dependencies = Vec::new();
        for task_id in task_id_iter {
            dependencies.push(task_id.unwrap());
        }
        Some(dependencies)
    }
    fn children(&self) -> Option<Vec<TaskID>> {
        let query = "SELECT task.taskid FROM task WHERE parent=?";
        let mut stmt = self.db.conn.prepare(query).unwrap();
        let task_id_iter = stmt.query_map([self.id], |row| {
                Ok(row.get(0)?)
            }).unwrap();

        let mut dependencies = Vec::new();
        for task_id in task_id_iter {
            dependencies.push(task_id.unwrap());
        }
        Some(dependencies)
    }

    fn serialize(&self, children: Vec<Task>) -> Task {
        Task::serialize(self, if children.len() > 0 { Some(children) } else { None }).unwrap()
    }
}

//Event
pub struct DatabaseEvent {
}

impl Event for DatabaseEvent {
    fn id(&self) -> EventID {
        todo!();
    }
    fn calendar(&self) -> CalendarID {
        todo!();
    }
    fn task(&self) -> TaskID {
        todo!();
    }
    fn from(&self) -> DateTime<Utc> {
        todo!();
    }
    fn until(&self) -> DateTime<Utc> {
        todo!();
    }
}

//Calendar
pub struct DatabaseCalendar<'a> {
    db: &'a Database,
    id: PrimaryKey,
}

// impl DatabaseCalendar {
// }

impl <'a> CalendarTrait for DatabaseCalendar<'a> {
    //Straight Properties
    fn calendar_id(&self) -> CalendarID {
        self.db.conn.query_row("SELECT calendarid FROM calendar WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn parent(&self) -> Option<CalendarID> {
        match self.db.conn.query_row("SELECT parent.calendarid FROM calendar child INNER JOIN calendar parent ON child.parent=parent.id WHERE child.id=?", [self.id], |r| r.get(0)) {
            Ok(task_id) => task_id,
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            _ => panic!(),
        }
    }
    fn algorithm(&self) -> Algorithm {
        self.db.conn.query_row("SELECT algorithm FROM calendar WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn from(&self) -> DateTime<Utc> {
        self.db.conn.query_row("SELECT from_ FROM calendar WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn until(&self) -> DateTime<Utc> {
        self.db.conn.query_row("SELECT until FROM calendar WHERE id=?", [self.id], |r| r.get(0)).unwrap()
    }
    fn value(&self) -> Option<f64> {
        match self.db.conn.query_row("SELECT value FROM plan WHERE calendar=? AND active=1", [self.id], |r| r.get(0)) {
            Ok(value) => value,
            // Err(rusqlite::Error::QueryReturnedNoRows) => None,
            _ => panic!(),
        }
    }

    fn current_event(&self)-> Option<EventID> {
        todo!();
    }
    fn current_task(&self) -> Option<TaskID> {
        todo!();
    }
    fn events(&self) -> Vec<EventID> {
        todo!();
    }

    // fn serialize(&self) -> Calendar {
    //     todo!();
    // }
    fn serialize(&self) -> Calendar {
        Calendar::serialize(self).unwrap()
    }
}

//PlanStatus
#[derive(Debug, Copy, Clone)]
pub enum PlanStatus {
    None,
    Preparing,
    Pending,
    Launching,//running
    InProcess,//running
    Incomplete,
    Complete,
    Inconsistent,
    Error,
}
impl ToSql for PlanStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            PlanStatus::None => Ok(1.into()),
            PlanStatus::Preparing => Ok(2.into()),
            PlanStatus::Pending => Ok(3.into()),
            PlanStatus::Launching => Ok(4.into()),
            PlanStatus::InProcess => Ok(5.into()),
            PlanStatus::Incomplete => Ok(6.into()),
            PlanStatus::Complete => Ok(7.into()),
            PlanStatus::Inconsistent => Ok(8.into()),
            PlanStatus::Error => Ok(9.into()),
        }
    }
}
impl FromSql for PlanStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(vi64) => {
                match vi64 {
                    1 => Ok(PlanStatus::None.into()),
                    2 => Ok(PlanStatus::Preparing.into()),
                    3 => Ok(PlanStatus::Pending.into()),
                    4 => Ok(PlanStatus::Launching.into()),
                    5 => Ok(PlanStatus::InProcess.into()),
                    6 => Ok(PlanStatus::Incomplete.into()),
                    7 => Ok(PlanStatus::Complete.into()),
                    8 => Ok(PlanStatus::Inconsistent.into()),
                    9 => Ok(PlanStatus::Error.into()),
                    _ => panic!("Unknown PlanStatus value: {}", vi64),
                }
            },
            _ => panic!("Unknown PlanStatus value: {:?}", value),
        }
    }
}
impl From<&PlanStatus> for u8 {
    fn from(c: &PlanStatus) -> Self {
        match c {
            PlanStatus::None => 1.into(),
            PlanStatus::Preparing => 2.into(),
            PlanStatus::Pending => 3.into(),
            PlanStatus::Launching => 4.into(),
            PlanStatus::InProcess => 5.into(),
            PlanStatus::Incomplete => 6.into(),
            PlanStatus::Complete => 7.into(),
            PlanStatus::Inconsistent => 8.into(),
            PlanStatus::Error => 9.into(),
        }
    }
}

//Proc
type ProcIndex = u16;
#[derive(Debug)]
struct ProcEvent {
    id: PrimaryKey,
    flexible: bool,
    start: Option<DateTime<Utc>>,
    due: Option<DateTime<Utc>>,
    scheduled: Option<DateTime<Utc>>,
    duration: Duration,
    dependencies: Vec<ProcIndex>,
}
#[derive(Debug)]
struct ProcPlan {
    id: PrimaryKey,
    deadline: Duration,
    events: Vec<ProcEvent>,
    events_set: usize,
    max_deps: usize,
}
#[derive(Debug, Copy, Clone)]
enum ProcOperationType {
    Insert,
    Swap,
}
#[derive(Debug, Copy, Clone)]
struct ProcOperation {
    op: ProcOperationType,
    from: ProcIndex,
    to: ProcIndex,
}
#[derive(Debug, Clone)]
struct ProcPlanUpdate {
    id: PrimaryKey,
    value : f64,
    set: Vec<ProcOperation>,
}

//Planner
impl Database {
    //Helper functions
    pub fn create_task(&self,
        parent: Option<PrimaryKey>,
        parent_taskid: Option<TaskID>,
        name: &String,
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
    ) -> Result<DatabaseTask,DataError> {
        //Create new
        let now = self.now()?;

        let task = Task::new(
            parent_taskid.clone(),
            now,//created
            name.clone(),
            task_type,
            status,
            macro_task_type,
            start,
            due,
            duration,
            body.clone(),
            meta1.clone(),
            meta2.clone(),
            meta3.clone(),
            value.clone(),
        )?;
        self.conn.execute(
            "INSERT INTO task (taskid,parent,created,name,tasktype,macrotasktype,status,start,due,duration,body,meta1,meta2,meta3,value) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![task.task_id,parent,now,name,task_type,macro_task_type,status,start,due,duration,body,meta1,meta2,meta3,value],
        )?;
        Ok(DatabaseTask {
            db: self,
            id: self.conn.last_insert_rowid(),
        })
    }

    //Private functions
    fn detect_cycle_recursive(conn: &Connection, top_task: PrimaryKey, node_task: PrimaryKey) -> bool {
        let query = "SELECT after FROM dependency WHERE before=? AND status=1";
        let mut stmt = conn.prepare(query).unwrap();
        let dependency_iter = stmt.query_map([node_task], |row| {
                Ok(row.get(0)?)
            }).unwrap();
        for after in dependency_iter {
            let after = after.unwrap();
            if top_task == after { return true; }
            if Database::detect_cycle_recursive(conn, top_task, after) { return true; }
        }
        return false;
    }
    fn create_calendar_plan(&self, algorithm: Algorithm, from: DateTime<Utc>, until: DateTime<Utc>, plan_status: PlanStatus) -> Result<(PrimaryKey, PrimaryKey),DataError> {
        let calendar = Calendar::new(
            None,
            algorithm,
            from,
            until,
            None,
        )?;

        self.conn.execute(
            "INSERT INTO calendar (calendarid,parent,algorithm,from_,until) VALUES (?,NULL,?,?,?)",
            params![calendar.calendar_id,algorithm,from,until],
        )?;
        let calendar = self.conn.last_insert_rowid();

        self.conn.execute(
            "INSERT INTO plan (calendar,active,value,status) VALUES (?,0,NULL,?)",
            params![calendar, plan_status],
        )?;
        let plan = self.conn.last_insert_rowid();
        Ok((calendar, plan))
    }
    fn fork_plan(&self, plan: PrimaryKey, plan_status: PlanStatus) -> Result<PrimaryKey,DataError> {
        self.conn.execute(
            "INSERT INTO plan (calendar,active,value,status) SELECT calendar,active,value,? FROM plan WHERE id=?",
            params![plan, plan_status],
        )?;
        let plan_forked = self.conn.last_insert_rowid();

        self.conn.execute(
            "INSERT INTO event (plan,task,from_,duration) SELECT ?,task,from_,duration FROM event WHERE plan=?",
            params![plan_forked,plan],
        )?;

        self.conn.execute(
            "INSERT INTO queue (plan,task) SELECT ?,task FROM queue WHERE plan=?",
            params![plan_forked,plan],
        )?;

        Ok(plan_forked)
    }
    fn delete_plan(&self, plan: PrimaryKey) -> Result<(),DataError> {
        self.conn.execute(
            "DELETE FROM event WHERE plan=?",
            params![plan],
        )?;

        self.conn.execute(
            "DELETE FROM plan WHERE id=?",
            params![plan],
        )?;

        Ok(())
    }
    fn evaluate_plan(&self, plan: PrimaryKey, update_value: bool) -> Result<bool,DataError> {
        let from: DateTime<Utc> = self.conn.query_row("SELECT calendar.from_ FROM plan INNER JOIN calendar ON plan.calendar=calendar.id WHERE plan.id=?", [plan.to_string()], |r| r.get(0)).unwrap();

        let query = "SELECT event.from_,event.duration,task.value FROM event INNER JOIN task ON event.task=task.id WHERE event.plan=? ORDER BY from_";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([plan], |row| {
            let plan_from : DateTime<Utc> = row.get(0)?;
            let duration : i64 = row.get(1)?;
            let value : f64 = row.get(2)?;

            Ok((plan_from, duration, value))
        }).unwrap();
        let mut now = from;
        let mut plan_value = 0f64;
        let mut has_value = false;
        let mut event_processor = |plan_from: DateTime<Utc>, duration: i64, value: f64| -> bool {
            if plan_from < now {
                return false;
            }
            now += chrono::Duration::seconds(duration);
            plan_value += value;
            has_value = true;

            true
        };
        for t in t_iter {
            let (plan_from, duration, value) = t.unwrap();

            if !event_processor(plan_from, duration, value) {
                return Ok(false)
            }
        }

        if update_value {
            if has_value {
                self.conn.execute(
                    "UPDATE plan SET value=? WHERE id=?",
                    params![plan_value,plan],
                )?;
            } else {
                self.conn.execute(
                    "UPDATE plan SET value=NULL WHERE id=?",
                    params![plan],
                )?;
            }
        }

        Ok(true)
    }

    fn choose_next_event_for_basic_plan(&self, plan: PrimaryKey, now: DateTime<Utc>) -> Result<Option<PrimaryKey>,DataError> {
        // let mut now = now;
        loop {
            let next: DateTime<Utc> = self.conn.query_row("SELECT MIN(from_) FROM event WHERE plan=? AND from_>?", [plan.to_string(), now.to_string()], |r| r.get(0)).unwrap();

            //3.2: Select from Queue: Task.Start <= NOW && NOW + Task.Duration <= NEXT
            let query = "SELECT queue.id,t.duration FROM queue INNER JOIN task t on queue.task=t.id WHERE plan=? AND (t.start IS NULL OR t.start<=?)";
            let mut stmt = self.conn
                .prepare(query).unwrap();
            //Decide if task duration is within available time (since Sqlite can't store duration for queries)
            let n_iter = stmt.query_map([plan.to_string(),now.to_string()], |row| {
                    let task_id : PrimaryKey = row.get(0)?;
                    let duration : Duration = row.get(1)?;
                    if now + duration.to_chrono_duration() <= next {
                        Ok(Some((task_id, duration)))
                    } else {
                        Ok(None)
                    }
                })?;
            let mut choices = Vec::new();
            for n in n_iter {
                if let Some((task_id, duration)) = n.unwrap() {
                    choices.push((task_id, duration));
                }
            }
            if choices.len() == 0 {
                //3.3: if no selection:
                //  NOW = first Event in Plan + duration; NEXT = next Event in Plan
                //  LOOP to 3.2
                let start_next: Option<DateTime<Utc>> = self.conn.query_row("SELECT MIN(task.start) FROM queue INNER JOIN task ON queue.task=task.id WHERE queue.plan=? AND task.start>?", [plan.to_string(), now.to_string()], |r| r.get(0)).unwrap();

                if let Some(start_next) = start_next {
                    panic!("{:?}\t{:?}\t{:?}\t{:?}", now, plan, next, start_next);
                } else {
                    let queue_count: PrimaryKey = self.conn.query_row("SELECT COUNT(*) FROM queue WHERE plan=?", [plan], |r| r.get(0)).unwrap();
                    
                    //3.6: if Queue empty:
                    //  calculate Value; return Success
                    if queue_count == 0 {
                        //Evaluate plan
                        let valid = self.evaluate_plan(plan, true)?;
                        if valid {
                            return Ok(Some(plan));
                        } else {
                            self.delete_plan(plan)?;
                            panic!("{:?}\t{:?}\t{:?}", now, plan, next);
                        }
                    } else {
                        panic!("{:?}\t{:?}\t{:?}", now, plan, next);

                        //3.7: else:
                        //  delete Plan; return Fail
                        self.delete_plan(plan)?;

                        todo!();
                    }
                }

                // now = cmp::min(next, start_next)

                todo!();
                // now = ?;
            } else {
                //3.5: else: For each T
                //  fork Plan
                //  move T from Queue to Plan [Event]
                //  increment NOW
                //  calculate NEXT
                //  validate Plan [for overlap & Due violations] ; return Fail
                //  LOOP to 3.2
                for (choice_task_id, duration) in &choices {
                    let forked_plan = self.fork_plan(plan, PlanStatus::None)?;

                    self.conn.execute(
                        "INSERT INTO event (plan,task,from_,duration) VALUES (?, ?, ?, ?)",
                        params![forked_plan, choice_task_id, now, duration],
                    )?;
                    self.conn.execute(
                        "DELETE FROM queue WHERE plan=? AND task=?",
                        params![forked_plan, choice_task_id],
                    )?;

                    let forked_now = now + duration.to_chrono_duration();

                    let forked_result = self.choose_next_event_for_basic_plan(forked_plan, forked_now).unwrap();
                    if let Some(forked_success) = forked_result {
                        self.delete_plan(plan)?;

                        return Ok(Some(forked_success));
                    }
                    else {
                        panic!("{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}", now, plan, choices, choice_task_id, forked_plan, forked_now, forked_result);
                    }
                }
                todo!();
            }
        }

        todo!();
    }

    fn create_calendar_trivial(&self, from: DateTime<Utc>, until: DateTime<Utc>) -> Result<Calendar,DataError> {
        //1: Create Calendar & empty Plan
        let (calendar, plan_default) = self.create_calendar_plan(Algorithm::Trivial, from, until, PlanStatus::None)?;

        let _plan_basic = self.create_calendar_basic(from, until, calendar, plan_default)?;

        let cal = DatabaseCalendar {
            db: self,
            id: calendar,
        };
        return Ok(cal.serialize());    
    }

    //INPUT: ProcPlan: ProcPlan==Valid, optional ProcEvent[] ; randomizer seed
    //OUTPUT: success ; ProcPlan.Value ; ProcPlanUpdate
    // OP:
    // type,ALPHA,BETA
    
    // OP:type=1: //Insert
    // -try_insert optional[ALPHA] at scheduled_idx[BETA]
    // -if scheduled_idx[BETA] past DEADLINE the FAIL
    // -if optional[ALPHA].duration <= break[BETA] then
    //     -if dependency violation then BETA++ && RETRY
    //     -else SUCCESS
    // -else [if optional[ALPHA].duration > break[BETA] then]
    //     -if !scheduled_idx[BETA].flexible then BETA++ && RETRY
    //     -else shift scheduled_idx[] up from BETA && set scheduled_idx[BETA] = ALPHA && recalculate break[] from BETA+1 && LOOP on X = BETA+1, Y = BETA+2:
    //         -if scheduled_idx[X] past DEADLINE the FAIL
    //         -if scheduled_idx[X].duration <= break[Y] then
    //             -if dependency violation then FAIL
    //             -else SUCCESS
    //         -else [if scheduled_idx[X].duration > break[X] then]
    //             -if !scheduled_idx[X].flexible then Y++ && RETRY LOOP
    //             -else X++ && RETRY LOOP

    // OP:type=1: //Swap

    fn process_plan_direct(plan: &ProcPlan, tx: Sender<Result<ProcPlanUpdate>>) {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let op = rng.gen_range(0..2);

        match op {
            0 => {
                todo!();
            },
            1 => {
                todo!();
            },
            _ => panic!("{:?}", op),
        }

        todo!();

        let update = ProcPlanUpdate {
            id: plan.id,
            value : 0f64,
            set: vec![],
        };

        tx.send(Ok(update)).unwrap();

        // todo!();
    }

    fn get_dependencies(&self, task: PrimaryKey) -> Vec<PrimaryKey> {
        let query = "SELECT before FROM dependency WHERE dependency.after=? AND dependency.status=1";
        let mut stmt = self.conn.prepare(query).unwrap();
        let task_id_iter = stmt.query_map([task], |row| {
                Ok(row.get(0)?)
            }).unwrap();

        let mut dependencies = Vec::new();
        for task_id in task_id_iter {
            dependencies.push(task_id.unwrap());
        }
        dependencies
    }

    fn get_procplan(&self, plan: PrimaryKey, from: DateTime<Utc>, until: DateTime<Utc>) -> ProcPlan {
        let query = "SELECT event.task,task.start,task.due,event.from_,event.duration,task.tasktype,task.value FROM event INNER JOIN task ON event.task=task.id WHERE event.plan=? ORDER BY from_";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([plan], |row| {
            let task : PrimaryKey = row.get(0)?;
            let start : Option<DateTime<Utc>> = row.get(1)?;
            let due : Option<DateTime<Utc>> = row.get(2)?;
            let event_from : DateTime<Utc> = row.get(3)?;
            let duration : Duration = row.get(4)?;
            let tasktype : TaskType = row.get(5)?;
            let value : f64 = row.get(6)?;

            Ok((task, start, due, event_from, duration, tasktype, value))
        }).unwrap();
        let mut max_deps = 0;
        let mut events = Vec::new();
        let mut event_index_map : HashMap<PrimaryKey, ProcIndex> = HashMap::new();        
        for t in t_iter {
            let (task, start, due, event_from, duration, tasktype, value) = t.unwrap();

            let dependencies = self.get_dependencies(task);
            let mut deps_indeces = Vec::with_capacity(dependencies.len());
            for dependency in dependencies {
                if !event_index_map.contains_key(&dependency) {
                    panic!("{:?}\t{:?}", event_index_map, dependency);
                } else {
                    let index = event_index_map[&dependency];
                    deps_indeces.push(index);
                }
            }
            max_deps = cmp::max(max_deps, deps_indeces.len());

            let proc_event = ProcEvent {
                id: task,
                flexible: tasktype != TaskType::Fixed,
                start: start,
                due: due,
                scheduled: Some(event_from),
                duration: duration,
                dependencies: deps_indeces,
            };

            events.push(proc_event);
            let event_index = events.len() - 1;

            event_index_map.insert(task, event_index.try_into().unwrap());
        }
        if events.len() == 0 {
            panic!("{:?}", plan);
        }

        let events_set = events.len();

        //We need to create an index of dependencies, but we can't guarantee that the order of the indices. So we'll keep looping until we have constructed a forward-only
        // array. (The alternative is to build the indices after the array, but this is easier for now.)
        loop {
            let mut skipped_reference = false;

            let query = "SELECT queue.task,task.start,task.due,task.duration,task.tasktype,task.value FROM queue INNER JOIN task ON queue.task=task.id WHERE queue.plan=?";
            let mut stmt = self.conn
                .prepare(query).unwrap();
            let t_iter = stmt.query_map([plan], |row| {
                let task : PrimaryKey = row.get(0)?;
                let start : Option<DateTime<Utc>> = row.get(1)?;
                let due : Option<DateTime<Utc>> = row.get(2)?;
                let duration : Duration = row.get(3)?;
                let tasktype : TaskType = row.get(4)?;
                let value : f64 = row.get(5)?;

                Ok((task, start, due, duration, tasktype, value))
            }).unwrap();
            for t in t_iter {
                let (task, start, due, duration, tasktype, value) = t.unwrap();

                if event_index_map.contains_key(&task) {
                    assert!(skipped_reference);
                } else {
                    let dependencies = self.get_dependencies(task);
                    let mut deps_indeces = Vec::with_capacity(dependencies.len());
                    let mut skipped_this_reference = false;
                    for dependency in dependencies {
                        if !event_index_map.contains_key(&dependency) {
                            skipped_reference = true;
                            skipped_this_reference = true;
                            panic!("{:?}\t{:?}", event_index_map, dependency);
                        } else {
                            let index = event_index_map[&dependency];
                            deps_indeces.push(index);
                        }
                    }

                    if !skipped_this_reference {
                        max_deps = cmp::max(max_deps, deps_indeces.len());

                        let proc_event = ProcEvent {
                            id: task,
                            flexible: tasktype != TaskType::Fixed,
                            start: start,
                            due: due,
                            scheduled: None,
                            duration: duration,
                            dependencies: deps_indeces,
                        };

                        events.push(proc_event);
                        let event_index = events.len() - 1;

                        event_index_map.insert(task, event_index.try_into().unwrap());
                    }
                }
            }
            if !skipped_reference {
                break;
            }
        }

        // panic!("{:?}\t{:?}", events, events_set);

        let plan = ProcPlan {
            id : plan,
            deadline : (from - until).into(),
            events : events,
            events_set : events_set,
            max_deps : max_deps,
        };

        return plan;
    }

    fn create_calendar_direct(&self, from: DateTime<Utc>, until: DateTime<Utc>) -> Result<Calendar,DataError> {
        //1: Create Calendar & empty Plan
        let (calendar, plan_default) = self.create_calendar_plan(Algorithm::Direct, from, until, PlanStatus::Preparing)?;

        let plan_basic = self.create_calendar_basic(from, until, calendar, plan_default)?;

        //4: OPTIONAL stage: Collect all open Tasks [not in Plan], then all dependent Tasks, etc.
        //4.0: Populate Queue: collect all open Tasks [not in Plan]
        //Add open Tasks directly
        self.conn.execute(
            "INSERT OR IGNORE INTO queue (plan,task) SELECT ?,id FROM task WHERE tasktype IN (2,3,4) AND status=2 AND (start IS NULL OR (start>=? AND start<=?))", // ORDER BY start
            params![plan_basic,from,until],
        )?;

        //4.0.1: Then collect all dependent Tasks, etc.
        let mut queue_count: PrimaryKey = self.conn.query_row("SELECT COUNT(*) FROM queue WHERE plan=?", [plan_basic], |r| r.get(0)).unwrap();
        loop {
            self.conn.execute(
                "INSERT OR IGNORE INTO queue (plan,task) SELECT ?,task.id FROM queue INNER JOIN dependency ON queue.task=dependency.after INNER JOIN task ON dependency.before=task.id WHERE dependency.status=1 AND task.tasktype IN (2,3,4) AND task.status=2 AND task.due<=?",
                params![plan_basic,until],
            )?;
            let count: PrimaryKey = self.conn.query_row("SELECT COUNT(*) FROM queue WHERE plan=?", [plan_basic], |r| r.get(0)).unwrap();
            if queue_count == count {
                break;
            }
            queue_count = count;
        }

        if queue_count == 0 {
            //No work to do!
            let valid = self.evaluate_plan(plan_basic, true)?;
            if !valid {
                self.delete_plan(plan_basic)?;
                return Err(DataError::InconsistentPlan);
            }

            //Set Complete
            self.conn.execute(
                "UPDATE plan SET status=7 WHERE id=?",
                params![plan_basic],
            )?;

            //6: return
            let cal = DatabaseCalendar {
                db: self,
                id: calendar,
            };
            return Ok(cal.serialize());
        } else {
            //4.0.2: Remove duplicates
            self.conn.execute(
                "DELETE FROM queue WHERE plan=? AND task IN (SELECT task FROM event WHERE plan=?)",
                params![plan_basic,plan_basic],
            )?;

            //Set Pending
            self.conn.execute(
                "UPDATE plan SET status=3 WHERE id=?",
                params![plan_basic],
            ).unwrap();
        }

        // Ready to create full calendar //

        //5: NON-SIGNALLING THREADS stage:
        loop {
            //5.1: Query for available Plans
            let plan_pending : Result<PrimaryKey> = self.conn.query_row("SELECT id FROM plan WHERE calendar=? AND status=3", [calendar], |r| r.get(0));
            match plan_pending {
                Ok(plan_pending) => {
                    //5.4: else if Pending Plan [AND AVAILABLE THREAD]:
                    // Launch thread for first available Plan
                    // LOOP to 5.1

                    //Set Launching
                    self.conn.execute(
                        "UPDATE plan SET status=4 WHERE id=?",
                        params![plan_pending],
                    ).unwrap();

                    let plan = self.get_procplan(plan_pending, from, until);

                    let (tx, rx): (Sender<Result<ProcPlanUpdate>>, Receiver<Result<ProcPlanUpdate>>) = channel();

                    let worker = thread::spawn(move || {
                        Self::process_plan_direct(&plan, tx);
                    });

                    //Set InProcess
                    self.conn.execute(
                        "UPDATE plan SET status=5 WHERE id=?",
                        params![plan_pending],
                    ).unwrap();

                    match rx.recv() {
                        Ok(result) => {
                            match result {
                                Ok(plan_update) => {
                                    let plan = self.get_procplan(plan_update.id, from, until);
                                    //let forked_plan = self.fork_plan(plan, PlanStatus::None)?;

                                    worker.join().unwrap();

                                    //Set Complete
                                    self.conn.execute(
                                        "UPDATE plan SET status=7 WHERE id=?",
                                        params![plan_update.id],
                                    ).unwrap();

                                    panic!("{:?}\t{:?}", plan, plan_update);
                                },
                                Err(_e) => {
                                    // Err(e)

                                    worker.join().unwrap();

                                    todo!();
                                },
                            }
                        },
                        Err(e) => {
                            panic!("{:?}", e);

                            worker.join().unwrap();

                            todo!();
                        },
                    }

                    todo!();
                },
                Err(rusqlite::Error::QueryReturnedNoRows) => 
                {
                    todo!();

                    //5.2: If no Pending OR running Plan, return highest value Plan...
                    let cal = DatabaseCalendar {
                        db: self,
                        id: calendar,
                    };
                    return Ok(cal.serialize());

                    //5.3: else if running but no Pending Plan:
                    // Wait for any thread to complete [DO NOT WAIT FOR TIMER]
                    // LOOP to 5.1
                    todo!();
                },
                _ => panic!(),
            }
        }
        todo!();
    }

    fn create_calendar_basic(&self, from: DateTime<Utc>, until: DateTime<Utc>, calendar: PrimaryKey, plan_default: PrimaryKey) -> Result<PrimaryKey,DataError> {
        //2: FIXED stage: Set all Fixed Tasks in Plan
        //Add Fixed events directly
        self.conn.execute(
            "INSERT INTO event (plan,task,from_,duration) SELECT ?,id,start,duration FROM task WHERE tasktype=6 AND status=2 AND start>=? AND start<=?",
            params![plan_default, from, until],
        )?;

        //Verify plan consistency
        let valid = self.evaluate_plan(plan_default, false)?;
        if !valid {
            self.delete_plan(plan_default)?;
            return Err(DataError::InconsistentPlan);
        }

        //2.1: Collect all dependent Tasks (from Fixed Tasks)
        self.conn.execute(
            "INSERT OR IGNORE INTO queue (plan,task) SELECT ?,task.id FROM event INNER JOIN dependency ON event.task=dependency.after INNER JOIN task ON dependency.before=task.id WHERE dependency.status=1 AND task.tasktype IN (2,3,4) AND task.status=2",
            params![plan_default],
        )?;

        //3: DUE stage:
        //3.0: Populate Queue: collect all Due Tasks [not in Plan]
        //Add Due events directly
        //ASSUME: (start IS NULL OR (start>=from AND start<=until))
        self.conn.execute(
            "INSERT INTO queue (plan,task) SELECT ?,id FROM task WHERE tasktype IN (2,3,4) AND status=2 AND due<=?", // ORDER BY start
            params![plan_default,until],
        )?;

        //3.0.1: Then collect all dependent Tasks, etc.
        let mut queue_count: PrimaryKey = self.conn.query_row("SELECT COUNT(*) FROM queue WHERE plan=?", [plan_default], |r| r.get(0)).unwrap();
        loop {
            self.conn.execute(
                "INSERT OR IGNORE INTO queue (plan,task) SELECT ?,task.id FROM queue INNER JOIN dependency ON queue.task=dependency.after INNER JOIN task ON dependency.before=task.id WHERE dependency.status=1 AND task.tasktype IN (2,3,4) AND task.status=2 AND task.due<=?",
                params![plan_default,until],
            )?;
            let count: PrimaryKey = self.conn.query_row("SELECT COUNT(*) FROM queue WHERE plan=?", [plan_default], |r| r.get(0)).unwrap();
            if queue_count == count {
                break;
            }
            queue_count = count;
        }
        if queue_count == 0 { //Empty or fixed calendar
            //Evaluate
            let valid = self.evaluate_plan(plan_default, true)?;
            if !valid {
                self.delete_plan(plan_default)?;
                return Err(DataError::InconsistentPlan);
            }

            //Set active
            self.conn.execute(
                "UPDATE plan SET active=1 WHERE id=?",
                params![plan_default],
            )?;

            return Ok(plan_default);
        }

        //3.1: NOW = from; NEXT = first Event in Plan
        let now = from;
        let viable_plan = self.choose_next_event_for_basic_plan(plan_default, now)?;
        if let Some(viable_plan) = viable_plan {
            self.delete_plan(plan_default)?;

            //Verify that there's exactly one valid plan
            let plan_count: PrimaryKey = self.conn.query_row("SELECT COUNT(*) FROM plan WHERE calendar=?", [calendar], |r| r.get(0)).unwrap();
            if plan_count != 1 {
                panic!("{:?}", plan_count);
                return Err(DataError::InconsistentPlan);
            }

            //4: return
            self.conn.execute(
                "UPDATE plan SET active=1 WHERE id=?",
                params![viable_plan],
            )?;

            return Ok(viable_plan);
        }
        else {
            panic!("{:?}", viable_plan);

            self.delete_plan(plan_default)?;
            return Err(DataError::InconsistentPlan);
        }
    }

    //Public functions
    pub fn new(file: bool) -> Result<Self> {
        let conn = if !file {
            Connection::open_in_memory()?
        } else {
            let path = "./my_db.db3";
            Connection::open(path)?
        };

        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS queue_email_unsorted (
                id          INTEGER PRIMARY KEY,
                date        DATETIME NOT NULL,
                message_id  TEXT NOT NULL UNIQUE,
                from_       TEXT NOT NULL,
                subject     TEXT NOT NULL,
                body_type   INTEGER NOT NULL,
                body        TEXT,
                text        TEXT
            );
            CREATE TABLE IF NOT EXISTS queue_github_notification (
                id          INTEGER PRIMARY KEY,
                date        DATETIME NOT NULL,
                repo        TEXT NOT NULL,
                thread      TEXT NOT NULL,
                from_       TEXT NOT NULL,
                body        TEXT,
                label       TEXT
            );
            CREATE TABLE IF NOT EXISTS task (
                id          INTEGER PRIMARY KEY,
                taskid      BLOB NOT NULL,
                parent      INTEGER,
                created     DATETIME NOT NULL,
                name        TEXT NOT NULL,
                tasktype    INTEGER NOT NULL,
                macrotasktype   INTEGER,
                status      INTEGER NOT NULL,
                start       DATETIME,
                due         DATETIME,
                duration    TIME,
                body        TEXT,
                meta1       TEXT,
                meta2       TEXT,
                meta3       TEXT,
                value       REAL,
                FOREIGN KEY(parent) REFERENCES task(id)
            );
            CREATE UNIQUE INDEX IF NOT EXISTS task_taskid ON task (taskid);
            CREATE TABLE IF NOT EXISTS dependency (
                before      INTEGER NOT NULL,
                after       INTEGER NOT NULL,
                status      INTEGER NOT NULL,
                FOREIGN KEY(before) REFERENCES task(id),
                FOREIGN KEY(after) REFERENCES task(id),
                CONSTRAINT dependency_before_neq_after CHECK (before <> after)
            );
            CREATE INDEX IF NOT EXISTS dependency_before ON dependency (before,status);
            CREATE INDEX IF NOT EXISTS dependency_after ON dependency (after,status);
            CREATE UNIQUE INDEX IF NOT EXISTS dependency_before_after ON dependency (before,after);
            CREATE UNIQUE INDEX IF NOT EXISTS dependency_before_after_status ON dependency (before,after,status);
            CREATE TABLE IF NOT EXISTS calendar (
                id          INTEGER PRIMARY KEY,
                calendarid  BLOB NOT NULL,
                parent      INTEGER,
                algorithm   INTEGER NOT NULL,
                from_       DATETIME NOT NULL,
                until       DATETIME NOT NULL,
                FOREIGN KEY(parent) REFERENCES calendar(id)
            );
            CREATE TABLE IF NOT EXISTS plan (
                id          INTEGER PRIMARY KEY,
                calendar    INTEGER NOT NULL,
                active      INTEGER NOT NULL,
                status      INTEGER NOT NULL,
                value       REAL,
                FOREIGN KEY(calendar) REFERENCES calendar(id)
            );
            CREATE INDEX IF NOT EXISTS plan_calendar ON plan (calendar);
            CREATE UNIQUE INDEX IF NOT EXISTS plan_calendar_active ON plan (calendar) WHERE active=1;
            CREATE TABLE IF NOT EXISTS event (
                id          INTEGER PRIMARY KEY,
                plan        INTEGER NOT NULL,
                task        INTEGER NOT NULL,
                from_       DATETIME NOT NULL,
                duration    TIME NOT NULL,
                FOREIGN KEY(plan) REFERENCES plan(id),
                FOREIGN KEY(task) REFERENCES task(id)
            );
            CREATE INDEX IF NOT EXISTS event_plan ON event (plan);
            CREATE UNIQUE INDEX IF NOT EXISTS event_plan_from ON event (plan,from_);
            CREATE INDEX IF NOT EXISTS event_task ON event (task);
            CREATE UNIQUE INDEX IF NOT EXISTS event_plan_task ON event (plan,task);
            CREATE TABLE IF NOT EXISTS queue (
                id          INTEGER PRIMARY KEY,
                plan        INTEGER NOT NULL,
                task        INTEGER NOT NULL,
                FOREIGN KEY(plan) REFERENCES plan(id),
                FOREIGN KEY(task) REFERENCES task(id)
            );
            CREATE INDEX IF NOT EXISTS queue_plan ON queue (plan);
            CREATE INDEX IF NOT EXISTS queue_task ON queue (task);
            CREATE UNIQUE INDEX IF NOT EXISTS queue_plan_task ON queue (plan,task);
            ",
        )?;

        Ok(Database {
            conn: conn,
        })
    }

    pub fn queue_email(&self, n: &EmailNotification) -> Result<(),DataError> {
        // println!("Email\t{:.99}", &n.message_id);
        let count_exists: PrimaryKey = self.conn.query_row("SELECT COUNT(*) FROM queue_email_unsorted WHERE message_id=?", [&n.message_id], |r| r.get(0)).unwrap();
        if count_exists > 0 {//Already queued
            // println!("Email\t{:.99}\t{}", &n.message_id, id);
            Ok(())
        } else {
            self.conn.execute(
                "INSERT INTO queue_email_unsorted (date,message_id,from_,subject,body_type,body,text) VALUES (?,?,?,?,?,?,?)",
                params![&n.date,&n.message_id,&n.from,&n.subject,n.body_type,&n.body,&n.text],
            )?;
            Ok(())
        }
    }
    pub fn process_queued_email<F>(&self, mut f: F) -> Result<(),DataError>  where F: FnMut(&EmailNotification) -> RowProcess {
        let query = "SELECT id,date,message_id,from_,subject,body_type,body,text FROM queue_email_unsorted";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let n_iter = stmt.query_map([], |row| {
                let id : PrimaryKey = row.get(0)?;
                Ok((EmailNotification {
                    date: row.get(1)?,
                    message_id: row.get(2)?,
                    from: row.get(3)?,
                    subject: row.get(4)?,
                    body_type: row.get(5)?,
                    body: row.get(6)?,
                    text: row.get(7)?,
                },id))
            })?;

        let mut deletes = Vec::new();
        for n in n_iter {
            let (n,id) = n.unwrap();
            let row_process = f(&n);
            match row_process {
                RowProcess::Leave => {},
                RowProcess::Delete => {
                    deletes.push(id);
                },
            }
        }
        if deletes.len() > 0 {
            let mut statement = "DELETE FROM queue_email_unsorted WHERE id IN (".to_string();
            for id in deletes {
                statement.push_str(&id.to_string());
                statement.push_str(",");
            }
            statement.pop();
            statement.push_str(")");
            self.conn.execute(&statement,())?;
        }

        Ok(())
    }
    pub fn queue_github_notification(&self, n: &GitHubNotification) -> Result<(),DataError> {
        self.conn.execute(
            "INSERT INTO queue_github_notification (date,repo,thread,from_,body,label) VALUES (?,?,?,?,?,?)",
            params![&n.date,&n.repo,&n.thread,&n.from,&n.body,&n.label],
        )?;
        Ok(())
    }
    pub fn process_queued_github_notifiction<F>(&self, mut f: F) -> Result<(),DataError> where F: FnMut(&GitHubNotification) -> RowProcess {
        let query = "SELECT id,date,repo,thread,from_,body,label FROM queue_github_notification";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let n_iter = stmt.query_map([], |row| {
                let id : PrimaryKey = row.get(0)?;
                Ok((GitHubNotification {
                    date: row.get(1)?,
                    repo: row.get(2)?,
                    thread: row.get(3)?,
                    from: row.get(4)?,
                    body: row.get(5)?,
                    label: row.get(6)?,
                },id))
            })?;

        let mut deletes = Vec::new();
        for n in n_iter {
            let (n,id) = n.unwrap();
            let row_process = f(&n);
            match row_process {
                RowProcess::Leave => {},
                RowProcess::Delete => {
                    deletes.push(id);
                },
            }
        }
        if deletes.len() > 0 {
            let mut statement = "DELETE FROM queue_github_notification WHERE id IN (".to_string();
            for id in deletes {
                statement.push_str(&id.to_string());
                statement.push_str(",");
            }
            statement.pop();
            statement.push_str(")");
            self.conn.execute(&statement,())?;
        }

        Ok(())
    }
    //https://github.com/rusqlite/rusqlite/issues/777
    pub fn get_macro_task_active_1(&self, name: &String, macro_task: Option<&DatabaseTask>, mtt: MacroTaskType, meta1: &str) -> Result<DatabaseTask,DataError> {
        if let Some(parent) = macro_task {
            //Get currently active
            if let Ok(id) = self.conn.query_row(
                "SELECT id FROM task WHERE parent=? AND macrotasktype=? AND meta1=? AND status=2 AND (start IS NULL OR start<=datetime('now'))",
                params![parent.id, mtt, meta1],
                |r| r.get(0)) {
                Ok(DatabaseTask {
                    db: self,
                    id: id,
                })
            } else {
                self.create_task(
                    Some(parent.id),//parent
                    Some(parent.task_id()),//parent_taskid
                    name,//name
                    TaskType::Macro,//task_type
                    Status::Active,//status
                    Some(mtt),//macro_task_type
                    None,//start
                    None,//due
                    None,//duration
                    None,//body
                    Some(meta1.to_string()),//meta1
                    None,//meta2
                    None,//meta3
                    None,//value
                )
            }
        } else {
            //Get currently active
            if let Ok(id) = self.conn.query_row(
                "SELECT id FROM task WHERE macrotasktype=? AND meta1=? AND status=2 AND (start IS NULL OR start<=datetime('now'))",
                params![mtt, meta1],
                |r| r.get(0)) {
                Ok(DatabaseTask {
                    db: self,
                    id: id,
                })
            } else {
                self.create_task(
                    None,//parent
                    None,//parent_taskid
                    name,//name
                    TaskType::Macro,//task_type
                    Status::Active,//status
                    Some(mtt),//macro_task_type
                    None,//start
                    None,//due
                    None,//duration
                    None,//body
                    Some(meta1.to_string()),//meta1
                    None,//meta2
                    None,//meta3
                    None,//value
                )
            }
        }
    }
    pub fn get_macro_task_active_2(&self, name: &String, macro_task: Option<&DatabaseTask>, mtt: MacroTaskType, meta1: &str, meta2: &str) -> Result<DatabaseTask,DataError> {
        if let Some(parent) = macro_task {
            //Get currently active
            if let Ok(id) = self.conn.query_row(
                "SELECT id FROM task WHERE parent=? AND macrotasktype=? AND meta1=? AND meta2=? AND status=2 AND (start IS NULL OR start<=datetime('now'))",
                params![parent.id, mtt, meta1, meta2],
                |r| r.get(0)) {
                Ok(DatabaseTask {
                    db: self,
                    id: id,
                })
            } else {
                self.create_task(
                    Some(parent.id),//parent
                    Some(parent.task_id()),//parent_taskid
                    name,//name
                    TaskType::Macro,//task_type
                    Status::Active,//status
                    Some(mtt),//macro_task_type
                    None,//start
                    None,//due
                    None,//duration
                    None,//body
                    Some(meta1.to_string()),//meta1
                    Some(meta2.to_string()),//meta2
                    None,//meta3
                    None,//value
                )
            }
        } else {
            //Get currently active
            if let Ok(id) = self.conn.query_row(
                "SELECT id FROM task WHERE macrotasktype=? AND meta1=? AND meta2=? AND status=2 AND (start IS NULL OR start<=datetime('now'))",
                params![mtt, meta1, meta2],
                |r| r.get(0)) {
                Ok(DatabaseTask {
                    db: self,
                    id: id,
                })
            } else {
                self.create_task(
                    None,//parent
                    None,//parent_taskid
                    name,//name
                    TaskType::Macro,//task_type
                    Status::Active,//status
                    Some(mtt),//macro_task_type
                    None,//start
                    None,//due
                    None,//duration
                    None,//body
                    Some(meta1.to_string()),//meta1
                    Some(meta2.to_string()),//meta2
                    None,//meta3
                    None,//value
                )
            }
        }
    }
    pub fn create_simple_task(&self, name: &String, due: Option<DateTime<Utc>>, body: &str) -> Result<DatabaseTask,DataError> {
        self.create_task(
            None,//parent
            None,//parent_taskid
            name,//name
            TaskType::Action,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            due,//due
            None,//duration
            Some(body.to_string()),//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        )
    }
    pub fn create_micro_task(&self, name: &String, macro_task: &DatabaseTask, date: &DateTime<Utc>, meta1: &str, body: &str) -> Result<DatabaseTask,DataError> {
        self.create_task(
            Some(macro_task.id),//parent
            Some(macro_task.task_id()),//parent_taskid
            name,//name
            TaskType::Micro,//task_type
            Status::Active,//status
            None,//macro_task_type
            Some(date.clone()),//start
            None,//due
            None,//duration
            Some(body.to_string()),//body
            Some(meta1.to_string()),//meta1
            None,//meta2
            None,//meta3
            None,//value
        )
    }
    pub fn iter_children<F>(&self, t: &DatabaseTask, mut f: F) -> Result<()>  where F: FnMut(&DatabaseTask) -> Result<()> {
        let query = "SELECT id FROM task WHERE parent=?";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([t.id], |row| {
                Ok(row.get(0)?)
            })?;

        for t in t_iter {
            f(&DatabaseTask {
                db: self,
                id: t.unwrap(),
            }).unwrap();
        }

        Ok(())
    }
    pub fn iter_active_children<F>(&self, t: &DatabaseTask, mut f: F) -> Result<()>  where F: FnMut(&DatabaseTask) -> Result<()> {
        let query = "SELECT id FROM task WHERE status=2 AND parent=? AND (start IS NULL OR start<=datetime('now')) ORDER BY value,start";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([t.id], |row| {
                Ok(row.get(0)?)
            })?;

        for t in t_iter {
            f(&DatabaseTask {
                db: self,
                id: t.unwrap(),
            }).unwrap();
        }

        Ok(())
    }
    pub fn iter_open_children<F>(&self, t: &DatabaseTask, mut f: F) -> Result<()>  where F: FnMut(&DatabaseTask) -> Result<()> {
        let query = "SELECT id FROM task WHERE status IN (1,2) AND parent=? ORDER BY value,start";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([t.id], |row| {
                Ok(row.get(0)?)
            })?;

        for t in t_iter {
            f(&DatabaseTask {
                db: self,
                id: t.unwrap(),
            }).unwrap();
        }

        Ok(())
    }
    pub fn iter_active_macro_tasks_order_1<F>(&self, t: &DatabaseTask, mut f: F) -> Result<()> where F: FnMut(&DatabaseTask) -> Result<()> {
        // let query = "SELECT id FROM task WHERE tasktype=4 AND parent IS NULL AND status=2 AND (start IS NULL OR start<=datetime('now'))";
        let query = "SELECT macro.id,MIN(micro.start) FROM task macro INNER JOIN task micro ON micro.parent=macro.id WHERE macro.tasktype=4 AND macro.parent=? AND macro.status=2 AND (macro.start IS NULL OR macro.start<=datetime('now')) AND micro.tasktype=5 AND micro.status=2 AND (micro.start IS NULL OR micro.start<=datetime('now')) GROUP BY macro.id ORDER BY 2";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([t.id], |row| {
                Ok(row.get(0)?)
            })?;

        for t in t_iter {
            f(&DatabaseTask {
                db: self,
                id: t.unwrap(),
            }).unwrap();
        }

        Ok(())
    }
    pub fn iter_active_top_tasks<F>(&self, mut f: F, limit: u32) -> Result<()> where F: FnMut(&DatabaseTask) -> Result<()> {
        //Get currently active
        let query = "SELECT id FROM task WHERE status=2 AND parent IS NULL AND (start IS NULL OR start<=datetime('now')) ORDER BY value,start LIMIT ?";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([limit], |row| {
                Ok(row.get(0)?)
            })?;

        for t in t_iter {
            f(&DatabaseTask {
                db: self,
                id: t.unwrap(),
            }).unwrap();
        }

        Ok(())
    }
    pub fn iter_open_top_tasks<F>(&self, mut f: F, limit: u32) -> Result<()> where F: FnMut(&DatabaseTask) -> Result<()> {
        //Get currently open
        let query = "SELECT id FROM task WHERE status IN (1,2) AND parent IS NULL ORDER BY value,start LIMIT ?";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([limit], |row| {
                Ok(row.get(0)?)
            })?;

        for t in t_iter {
            f(&DatabaseTask {
                db: self,
                id: t.unwrap(),
            }).unwrap();
        }

        Ok(())
    }
    pub fn iter_available_tasks<F>(&self, mut f: F, limit: u32) -> Result<()> where F: FnMut(&DatabaseTask) -> Result<()> {
        //Get currently available
        // let query = "SELECT id FROM task WHERE status IN (1,2) AND parent IS NULL AND (start IS NULL OR start<=datetime('now')) AND id NOT IN (SELECT after FROM dependency WHERE status=1) ORDER BY start";
        let query = "SELECT id FROM task t LEFT OUTER JOIN dependency d ON after=id WHERE t.status IN (1,2) AND parent IS NULL AND (start IS NULL OR start<=datetime('now')) AND id NOT IN (SELECT after FROM dependency WHERE status=1) AND after IS NULL ORDER BY value,start LIMIT ?";
        let mut stmt = self.conn
            .prepare(query).unwrap();
        let t_iter = stmt.query_map([limit], |row| {
                Ok(row.get(0)?)
            })?;

        for t in t_iter {
            f(&DatabaseTask {
                db: self,
                id: t.unwrap(),
            }).unwrap();
        }

        Ok(())
    }

    fn serialize_with_active_children(&self, task: &DatabaseTask) -> Task {
        //Serialize children recursively
        let mut child_tasks: Vec<Task> = Vec::new();
        let active_child_task_processor = |child_task: &DatabaseTask| {
            child_tasks.push(self.serialize_with_active_children(child_task));
            Ok(())
        };
        self.iter_active_children(task, active_child_task_processor).unwrap();

        //Serialize with children
        task.serialize(child_tasks)
    }
    fn serialize_with_open_children(&self, task: &DatabaseTask) -> Task {
        //Serialize children recursively
        let mut child_tasks: Vec<Task> = Vec::new();
        let open_child_task_processor = |child_task: &DatabaseTask| {
            child_tasks.push(self.serialize_with_open_children(child_task));
            Ok(())
        };
        self.iter_open_children(task, open_child_task_processor).unwrap();

        //Serialize with children
        task.serialize(child_tasks)
    }
}

impl Planner for Database {
    fn now(&self) -> Result<DateTime<Utc>,DataError> {
        match self.conn.query_row("SELECT datetime('now')", [], |r| r.get(0)) {
            Ok(now) => Ok(now),
            Err(e) => Err(DataError::RusqliteError(e)),
        }
    }

    fn detect_dependency_cycles(&self) -> Result<bool,DataError> {
        //Can start 1 level deep since self-references are not possible
        let query = "SELECT L.before,L.after,R.after FROM dependency L INNER JOIN dependency R ON L.after=R.before WHERE L.status=1 AND R.status=1";
        let mut stmt = self.conn.prepare(query).unwrap();
        let dependency_iter = stmt.query_map([], |row| {
                let before : PrimaryKey = row.get(0)?;
                let middle : PrimaryKey = row.get(1)?;
                let after : PrimaryKey = row.get(2)?;

                Ok((before, middle, after))
            }).unwrap();
        for before_middle_after in dependency_iter {
            let (before, _middle, after) = before_middle_after.unwrap();
            if before == after { return Ok(true); }
            if Database::detect_cycle_recursive(&self.conn, before, after) { return Ok(true); }
        }
        Ok(false)
    }

    fn get_task<'a,DatabaseTask>(&self, _id: TaskID) -> Result<DatabaseTask,DataError> {
        todo!();
    }
    fn get_event<'a,DatabaseEvent>(&self, _id: EventID) -> Result<DatabaseEvent,DataError> {
        todo!();
    }
    fn get_calendar(&self, _id: CalendarID) -> Result<Calendar,DataError> {
        todo!();
    }

    fn active_tasks(&self, limit: u32) -> Vec<Task> {
        //Collect all tasks
        let mut tasks = Vec::new();
        //TaskType
        let active_task_processor = |task: &DatabaseTask| {
            tasks.push(self.serialize_with_active_children(task));
            Ok(())
        };
        self.iter_active_top_tasks(active_task_processor, limit).unwrap();

        tasks
    }
    fn open_tasks(&self, limit: u32) -> Vec<Task> {
        //Collect all tasks
        let mut tasks = Vec::new();
        //TaskType
        let open_task_processor = |task: &DatabaseTask| {
            tasks.push(self.serialize_with_open_children(task));
            Ok(())
        };
        self.iter_open_top_tasks(open_task_processor, limit).unwrap();

        tasks
    }
    fn available_tasks(&self, limit: u32) -> Vec<Task> {
        //Collect all tasks
        let mut tasks = Vec::new();
        //TaskType
        let available_task_processor = |task: &DatabaseTask| {
            tasks.push(self.serialize_with_open_children(task));
            Ok(())
        };
        self.iter_available_tasks(available_task_processor, limit).unwrap();

        tasks
    }
    fn set_dependency(&self, before: &TaskID, after: &TaskID) -> Result<(),DataError> {
        //"INSERT INTO dependency (before,after,status) SELECT b.id,a.id,1 FROM task b CROSS JOIN task a WHERE b.taskid=? AND a.taskid=?",
        let before_id : PrimaryKey = self.conn.query_row("SELECT id FROM task WHERE taskid=?", [before], |r| r.get(0))?;
        let after_id : PrimaryKey = self.conn.query_row("SELECT id FROM task WHERE taskid=?", [after], |r| r.get(0))?;

        //Check self-reference
        if before_id == after_id { return Err(DataError::CircularDependency) }
        //Check cycle
        if Database::detect_cycle_recursive(&self.conn, before_id, after_id) { return Err(DataError::CircularDependency) }

        //Insert
        self.conn.execute(
            "INSERT INTO dependency (before,after,status) VALUES (?,?,1)",
            params![before_id,after_id],
        )?;
        assert_ne!(self.conn.last_insert_rowid(), 0);        
        Ok(())
    }
    fn unset_dependency(&self, _before: &TaskID, _after: &TaskID) -> Result<(),DataError> {
        todo!();
    }

    fn create_calendar(&self, algorithm: Algorithm, from: DateTime<Utc>, until: DateTime<Utc>) -> Result<Calendar,DataError> {
        match algorithm {
            Algorithm::Trivial => return self.create_calendar_trivial(from, until),
            Algorithm::Direct => return self.create_calendar_direct(from, until),
        }
        // todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_new() {
        let _ = Database::new(false).unwrap();
    }

    #[test]
    fn database_now() {
        let db = Database::new(false).unwrap();
        let _ = db.now().unwrap();
    }

    #[test]
    fn database_detect_dependency_cycles() {
        let db = Database::new(false).unwrap();

        let before = db.create_task(
            None,//parent
            None,//parent_taskid
            &"before".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let middle = db.create_task(
            None,//parent
            None,//parent_taskid
            &"middle".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let after = db.create_task(
            None,//parent
            None,//parent_taskid
            &"after".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        db.set_dependency(&before.task_id(), &middle.task_id()).unwrap();

        let detected = db.detect_dependency_cycles().unwrap();
        assert_eq!(detected, false);

        db.set_dependency(&middle.task_id(), &after.task_id()).unwrap();

        let detected = db.detect_dependency_cycles().unwrap();
        assert_eq!(detected, false);

        // db.set_dependency(&before.task_id(), &after.task_id()).unwrap();
    
        let result = db.set_dependency(&middle.task_id(), &before.task_id());
        assert!(matches!(result, Err(DataError::CircularDependency)));

        let detected = db.detect_dependency_cycles().unwrap();
        assert_eq!(detected, false);

        let result = db.set_dependency(&after.task_id(), &middle.task_id());
        assert!(matches!(result, Err(DataError::CircularDependency)));

        let detected = db.detect_dependency_cycles().unwrap();
        assert_eq!(detected, false);

        let result = db.set_dependency(&after.task_id(), &before.task_id());
        assert!(matches!(result, Err(DataError::CircularDependency)));

        // let detected = db.detect_dependency_cycles().unwrap();
        // assert_eq!(detected, true);
        let detected = db.detect_dependency_cycles().unwrap();
        assert_eq!(detected, false);
    }

    #[test]
    fn database_active_tasks_blank() {
        let db = Database::new(false).unwrap();

        let tasks = db.active_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn database_active_tasks() {
        let db = Database::new(false).unwrap();

        let _ = db.create_task(
            None,//parent
            None,//parent_taskid
            &"a".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let tasks = db.active_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn database_inactive_tasks() {
        let db = Database::new(false).unwrap();

        let _ = db.create_task(
            None,//parent
            None,//parent_taskid
            &"a".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            Some(chrono::offset::Utc::now() + chrono::Duration::days(100)),//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let tasks = db.active_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn database_open_tasks_blank() {
        let db = Database::new(false).unwrap();

        let tasks = db.open_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn database_open_tasks() {
        let db = Database::new(false).unwrap();

        let _ = db.create_task(
            None,//parent
            None,//parent_taskid
            &"a".to_string(),//name
            TaskType::Idea,//task_type
            Status::Pending,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let tasks = db.open_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn database_unopen_tasks() {
        let db = Database::new(false).unwrap();

        let _ = db.create_task(
            None,//parent
            None,//parent_taskid
            &"a".to_string(),//name
            TaskType::Idea,//task_type
            Status::Complete,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let tasks = db.open_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn database_available_tasks() {
        let db = Database::new(false).unwrap();

        let parent = db.create_task(
            None,//parent
            None,//parent_taskid
            &"parent".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let _ = db.create_task(
            Some(parent.id),//parent
            Some(parent.task_id()),//parent_taskid
            &"child".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let tasks = db.available_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn database_children() {
        let db = Database::new(false).unwrap();

        let parent = db.create_task(
            None,//parent
            None,//parent_taskid
            &"parent".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let _ = db.create_task(
            Some(parent.id),//parent
            Some(parent.task_id()),//parent_taskid
            &"child".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let children = parent.children().unwrap();
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn database_set_dependency() {
        let db = Database::new(false).unwrap();

        let before = db.create_task(
            None,//parent
            None,//parent_taskid
            &"before".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let after = db.create_task(
            None,//parent
            None,//parent_taskid
            &"after".to_string(),//name
            TaskType::Idea,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            None,//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            None,//value
        ).unwrap();

        let dependency_count : u32 = db.conn.query_row("SELECT COUNT(*) FROM dependency", [], |r| r.get(0)).unwrap();
        assert_eq!(dependency_count, 0);

        db.set_dependency(&before.task_id(), &after.task_id()).unwrap();

        let dependency_count : u32 = db.conn.query_row("SELECT COUNT(*) FROM dependency", [], |r| r.get(0)).unwrap();
        assert_eq!(dependency_count, 1);

        let tasks = db.available_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 1);
    }

    fn create_simple_task()
    {
        let db = Database::new(false).unwrap();

        let _ = db.create_simple_task(
            &"a".to_string(),//name
            None,//due
            "b",//body
        ).unwrap();

        let tasks = db.available_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn create_calendar_trivial_empty() {
        let db = Database::new(false).unwrap();
        let now = db.now().unwrap();
        let until = now + Days::new(1);

        let calendar = db.create_calendar(Algorithm::Trivial, now, until).unwrap();
        assert_eq!(calendar.value, None);
    }

    #[test]
    fn create_calendar_trivial_single() {
        let db = Database::new(false).unwrap();
        let now = db.now().unwrap();
        let until = now + Days::new(1);

        let _ = db.create_task(
            None,//parent
            None,//parent_taskid
            &"a".to_string(),//name
            TaskType::Fixed,//task_type
            Status::Active,//status
            None,//macro_task_type
            Some(now),//start
            None,//due
            Some(chrono::Duration::seconds(60i64).into()),//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            Some(1f64),//value
        ).unwrap();
        let tasks = db.open_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 1);

        let calendar = db.create_calendar(Algorithm::Trivial, now, until).unwrap();
        assert_eq!(calendar.value, Some(1f64));
    }

    #[test]
    fn create_calendar_trivial_ordered() {
        let db = Database::new(false).unwrap();
        let now = db.now().unwrap();
        let until = now + Days::new(2);
        let due = now + Days::new(1);

        let before = db.create_task(
            None,//parent
            None,//parent_taskid
            &"before".to_string(),//name
            TaskType::Action,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            Some(chrono::Duration::seconds(60i64).into()),//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            Some(1f64),//value
        ).unwrap();

        let after = db.create_task(
            None,//parent
            None,//parent_taskid
            &"after".to_string(),//name
            TaskType::Fixed,//task_type
            Status::Active,//status
            None,//macro_task_type
            Some(due),//start
            Some(due),//due
            Some(chrono::Duration::seconds(60i64).into()),//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            Some(2f64),//value
        ).unwrap();

        let dependency_count : u32 = db.conn.query_row("SELECT COUNT(*) FROM dependency", [], |r| r.get(0)).unwrap();
        assert_eq!(dependency_count, 0);

        db.set_dependency(&before.task_id(), &after.task_id()).unwrap();

        let calendar = db.create_calendar(Algorithm::Trivial, now, until).unwrap();
        assert_eq!(calendar.value, Some(3f64));
    }

    #[test]
    fn create_calendar_direct_empty() {
        let db = Database::new(false).unwrap();
        let now = db.now().unwrap();
        let until = now + Days::new(1);

        let calendar = db.create_calendar(Algorithm::Direct, now, until).unwrap();
        assert_eq!(calendar.value, None);
    }

    #[test]
    fn create_calendar_direct_single() {
        let db = Database::new(false).unwrap();
        let now = db.now().unwrap();
        let until = now + Days::new(1);

        let _ = db.create_task(
            None,//parent
            None,//parent_taskid
            &"a".to_string(),//name
            TaskType::Fixed,//task_type
            Status::Active,//status
            None,//macro_task_type
            Some(now),//start
            None,//due
            Some(chrono::Duration::seconds(60i64).into()),//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            Some(1f64),//value
        ).unwrap();
        let tasks = db.open_tasks(std::u32::MAX);
        assert_eq!(tasks.len(), 1);

        let calendar = db.create_calendar(Algorithm::Direct, now, until).unwrap();
        assert_eq!(calendar.value, Some(1f64));
    }

    #[test]
    fn create_calendar_direct_ordered() {
        let db = Database::new(false).unwrap();
        let now = db.now().unwrap();
        let until = now + Days::new(2);
        let due = now + Days::new(1);

        let before = db.create_task(
            None,//parent
            None,//parent_taskid
            &"before".to_string(),//name
            TaskType::Action,//task_type
            Status::Active,//status
            None,//macro_task_type
            None,//start
            None,//due
            Some(chrono::Duration::seconds(60i64).into()),//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            Some(1f64),//value
        ).unwrap();

        let after = db.create_task(
            None,//parent
            None,//parent_taskid
            &"after".to_string(),//name
            TaskType::Fixed,//task_type
            Status::Active,//status
            None,//macro_task_type
            Some(due),//start
            Some(due),//due
            Some(chrono::Duration::seconds(60i64).into()),//duration
            None,//body
            None,//meta1
            None,//meta2
            None,//meta3
            Some(2f64),//value
        ).unwrap();

        let dependency_count : u32 = db.conn.query_row("SELECT COUNT(*) FROM dependency", [], |r| r.get(0)).unwrap();
        assert_eq!(dependency_count, 0);

        db.set_dependency(&before.task_id(), &after.task_id()).unwrap();

        let calendar = db.create_calendar(Algorithm::Direct, now, until).unwrap();
        assert_eq!(calendar.value, Some(3f64));
    }
}