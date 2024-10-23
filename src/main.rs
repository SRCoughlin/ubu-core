#![allow(deprecated)]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]

use regex::Regex;
mod pop3client;
mod datamodel;
mod rest;
use crate::datamodel::data::*;
use crate::datamodel::database::*;
use std::env;
use warp::Filter;
use std::collections::HashMap;
use chrono::{Utc, DateTime};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let db;
    if cfg!(not(any(feature = "json",feature = "process"))) {
        db = Database::new(true).unwrap();
    } else {
        db = Database::new(false).unwrap();
    }


    if cfg!(not(any(feature = "json",feature = "process"))) {
        //
        // Sensors
        //

        //Get email notifications
        let notification_handler = |n: &EmailNotification| {
            db.queue_email(n).unwrap();
            pop3client::EmailProcess::Delete
        };
        match pop3client::get_email(notification_handler) {
            Err(why) => panic!("{:?}", why),
            Ok(_) => { },
        }
    }


    if cfg!(feature = "process") {
        //
        // Data Processing
        //

        //Process email notifications
        let label = "personal";
        let email_processor = |n: &EmailNotification| {
            //GitHub <noreply@github.com>
            //Tamir Hemo <notifications@github.com>
            let re = Regex::new(r"(.+) <notifications@github.com>$").unwrap();
            if let Some(caps) = re.captures(&n.from) {
                let from = &caps[1];

                //Re: [microsoft/Nova] Expose verifier key's digest with a trait (PR #243)
                let re = Regex::new(r"^(Re: )?\[(?<r>[^\]]+)\] (?<t>.+)$").unwrap();
                let Some(caps) = re.captures(&n.subject) else {
                    panic!();
                };
                let repo = &caps["r"];
                let thread = &caps["t"];

                let g = GitHubNotification {
                    date: n.date,
                    repo: repo.to_string(),
                    thread: thread.to_string(),
                    from: from.to_string(),
                    body: n.body.clone(),
                    label: label.to_string(),
                };

                db.queue_github_notification(&g).unwrap();

                RowProcess::Delete
            } else {
                todo!();
            }
        };
        db.process_queued_email(email_processor).unwrap();

        //Process GitHub notifications
        let macro_task_github = db.get_macro_task_active_1(&"GitHub".to_string(), None, MacroTaskType::GitHub, label).unwrap();
        println!("macro_task_github\t{:.50}\t{:.50}\t{:.50}\t{:.50}\t{:.50}", label, macro_task_github.created(), macro_task_github.status(), macro_task_github.body().unwrap_or("".to_string()), macro_task_github.meta1().unwrap_or("".to_string()));

        let github_notifiction_processor = |n: &GitHubNotification| {
            let macro_task_repo_thread = db.get_macro_task_active_2(&format!("{:.15},{:.15}", n.repo, n.thread), Some(&macro_task_github), MacroTaskType::GitHub, &n.repo, &n.thread).unwrap();

            let _micro_task = db.create_micro_task(&format!("{:.30}", n.body), &macro_task_repo_thread, &n.date, &n.from, &n.body).unwrap();

            RowProcess::Delete
        };
        db.process_queued_github_notifiction(github_notifiction_processor).unwrap();

        //Print macro tasks
        let active_macro_task_processor = |macro_t: &DatabaseTask| {
            assert_eq!(macro_t.task_type(),TaskType::Macro);
            let Some(mtt) = macro_t.macro_task_type() else {
                //Impossible code
                todo!();
            };
            match mtt {
                MacroTaskType::Email => todo!(),
                MacroTaskType::GitHub => {
                    let repo = macro_t.meta1().unwrap();
                    let thread = macro_t.meta2().unwrap();

                    db.iter_active_children(macro_t, |micro: &DatabaseTask| {
                        assert_eq!(micro.task_type(),TaskType::Micro);
                        let start = micro.start().unwrap();
                        let body = micro.body().unwrap_or("".to_string());

                        println!("GitHub\t{:.50}\t{:.50}\t{:.50}\t{:.50}\t{:.1}", label, repo, thread, start, body);

                        Ok(())
                    }).unwrap();
                    Ok(())
                },
            }
        };
        db.iter_active_macro_tasks_order_1(&macro_task_github, active_macro_task_processor).unwrap();
    }


    if cfg!(feature = "json") {
        // // EXAMPLES:
        // // https://github.com/blurbyte/restful-rust
        // // https://github.com/blurbyte/restful-rust/blob/master/src/main.rs
        // // Maybe: https://hub.qovery.com/guides/tutorial/create-a-blazingly-fast-api-in-rust-part-1/
        // // REST API
        // // Show debug logs by default by setting `RUST_LOG=restful_rust=debug`
        // if env::var_os("RUST_LOG").is_none() {
        //     env::set_var("RUST_LOG", "restful_rust=debug");
        // }
        // pretty_env_logger::init();


        let _ = db.create_simple_task(
            &"name".to_string(),//name
            None,//due
            "body",//body
        ).unwrap();

        let db = Arc::new(Mutex::new(db));

        let api = rest::routes::tasks_routes(db);

        let routes = api.with(warp::log("restful_rust"));

        // Start the server
        // http://127.0.0.1:8081/tasks?limit=5
        println!("Starting the server...");
        warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
    }
}