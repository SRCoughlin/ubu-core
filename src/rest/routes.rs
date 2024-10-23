// Provides RESTful API for tasks:
//
// - `GET /tasks`: return JSON list of tasks
// - `POST /tasks`: create a new task entry
// - `PUT /tasks/:id`: update a specific task
// - `DELETE /tasks/:id`: delete a specific task

use warp::{Filter, Rejection, Reply};

use crate::rest::custom_filters;
use crate::rest::handlers;
use crate::rest::schema::Db;

// Root, all routes combined
pub fn tasks_routes(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    tasks_list(db.clone())
        // .or(tasks_create(db.clone()))
        // .or(tasks_update(db.clone()))
        // .or(tasks_delete(db))
}

// `GET /tasks?limit=5`
pub fn tasks_list(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("tasks")
        .and(warp::get())
        .and(custom_filters::list_options())
        .and(custom_filters::with_db(db))
        .and_then(handlers::list_tasks)
}

// // `POST /tasks`
// pub fn tasks_create(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
//     warp::path!("tasks")
//         .and(warp::post())
//         .and(custom_filters::json_body())
//         .and(custom_filters::with_db(db))
//         .and_then(handlers::create_task)
// }

// // `PUT /tasks/:id`
// pub fn tasks_update(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
//     warp::path!("tasks" / u64)
//         .and(warp::put())
//         .and(custom_filters::json_body())
//         .and(custom_filters::with_db(db))
//         .and_then(handlers::update_task)
// }

// // `DELETE /tasks/:id`
// pub fn tasks_delete(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
//     warp::path!("tasks" / u64)
//         .and(warp::delete())
//         .and(custom_filters::with_db(db))
//         .and_then(handlers::delete_task)
// }

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::prelude::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use warp::http::StatusCode;

    use crate::datamodel::database::*;
    use crate::datamodel::data::*;
    use crate::rest::schema::{/*Game,*/ Genre};

    // Mocked dataset for each test

    // fn mocked_db() -> Db {
    //     Arc::new(Mutex::new(vec![
    //         Game {
    //             id: 1,
    //             title: String::from("Crappy title"),
    //             rating: 35,
    //             genre: Genre::RolePlaying,
    //             description: Some(String::from("Test description...")),
    //             release_date: NaiveDate::from_ymd(2011, 9, 22).and_hms(0, 0, 0),
    //         },
    //         Game {
    //             id: 2,
    //             title: String::from("Decent game"),
    //             rating: 84,
    //             genre: Genre::Strategy,
    //             description: None,
    //             release_date: NaiveDate::from_ymd(2014, 3, 11).and_hms(0, 0, 0),
    //         },
    //     ]))
    // }
    // fn mocked_db() -> Db {
    //     let db = Database::new(false).unwrap();

    //     Arc::new(Mutex::new(db))
    // }

    // fn mocked_game() -> Game {
    //     Game {
    //         id: 3,
    //         title: String::from("Another game"),
    //         rating: 65,
    //         description: None,
    //         genre: Genre::Strategy,
    //         release_date: NaiveDate::from_ymd(2016, 3, 11).and_hms(0, 0, 0),
    //     }
    // }

    #[tokio::test]
    async fn get_tasks_empty() {
        let db = Database::new(false).unwrap();
        let filter = tasks_routes(Arc::new(Mutex::new(db)));

        let res = warp::test::request().method("GET").path("/tasks").reply(&filter).await;

        assert_eq!(res.status(), StatusCode::OK);

        let expected_json_body = r#"[]"#;
        assert_eq!(res.body(), expected_json_body);
    }

    #[tokio::test]
    async fn get_tasks_single() {
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

        let filter = tasks_routes(Arc::new(Mutex::new(db)));

        let res = warp::test::request().method("GET").path("/tasks").reply(&filter).await;

        assert_eq!(res.status(), StatusCode::OK);

        let tasks: Vec<Task> = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn get_tasks_100() {
        let db = Database::new(false).unwrap();

        for i in 0..100 {
            let s: String = i.to_string();
            let _ = db.create_task(
                None,//parent
                None,//parent_taskid
                &s,//name
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
        }

        let filter = tasks_routes(Arc::new(Mutex::new(db)));

        let res = warp::test::request().method("GET").path("/tasks").reply(&filter).await;

        assert_eq!(res.status(), StatusCode::OK);

        let tasks: Vec<Task> = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(tasks.len(), 100);
    }

    #[tokio::test]
    async fn get_tasks_with_options() {
        let db = Database::new(false).unwrap();

        for i in 0..6 {
            let s: String = i.to_string();
            let _ = db.create_task(
                None,//parent
                None,//parent_taskid
                &s,//name
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
        }

        let filter = tasks_routes(Arc::new(Mutex::new(db)));

        let res = warp::test::request()
            .method("GET")
            .path("/tasks?limit=5")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::OK);

        let tasks: Vec<Task> = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(tasks.len(), 5);
    }

    #[tokio::test]
    async fn get_tasks_incorrect_options() {
        let db = Database::new(false).unwrap();
        let filter = tasks_routes(Arc::new(Mutex::new(db)));

        let res = warp::test::request()
            .method("GET")
            .path("/tasks?limit=x")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_tasks_not_found_404() {
        let db = Database::new(false).unwrap();
        let filter = tasks_routes(Arc::new(Mutex::new(db)));

        let res = warp::test::request()
            .method("GET")
            .path("/tasks/42")
            .reply(&filter)
            .await;

        // assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    // #[tokio::test]
    // async fn post_json_201() {
    //     let db = mocked_db();
    //     let filter = games_routes(db.clone());

    //     let res = warp::test::request()
    //         .method("POST")
    //         .json(&mocked_game())
    //         .path("/games")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::CREATED);
    //     assert_eq!(db.lock().await.len(), 3);
    // }

    // #[tokio::test]
    // async fn post_too_long_content_413() {
    //     let db = mocked_db();
    //     let filter = games_routes(db);

    //     let res = warp::test::request()
    //         .method("POST")
    //         .header("content-length", 1024 * 36)
    //         .path("/games")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    // }

    // #[tokio::test]
    // async fn post_wrong_payload_400() {
    //     let db = mocked_db();
    //     let filter = games_routes(db);

    //     let res = warp::test::request()
    //         .method("POST")
    //         .body(&r#"{"id":4}"#)
    //         .path("/games")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    // }

    // #[tokio::test]
    // async fn post_wrong_path_405() {
    //     let db = mocked_db();
    //     let filter = games_routes(db);

    //     let res = warp::test::request()
    //         .method("POST")
    //         .path("/games/42")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    // }

    // #[tokio::test]
    // async fn put_json_200() {
    //     let db = mocked_db();
    //     let filter = games_routes(db.clone());

    //     let res = warp::test::request()
    //         .method("PUT")
    //         .json(&mocked_game())
    //         .path("/games/2")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::OK);

    //     let db = db.lock().await;
    //     let ref title = db[1].title;
    //     assert_eq!(title, "Another game");
    // }

    // #[tokio::test]
    // async fn put_wrong_id_404() {
    //     let db = mocked_db();
    //     let filter = games_routes(db);

    //     let res = warp::test::request()
    //         .method("PUT")
    //         .json(&mocked_game())
    //         .path("/games/42")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::NOT_FOUND);
    // }

    // #[tokio::test]
    // async fn put_wrong_payload_400() {
    //     let db = mocked_db();
    //     let filter = games_routes(db);

    //     let res = warp::test::request()
    //         .method("PUT")
    //         .header("content-length", 1024 * 16)
    //         .body(&r#"{"id":2"#)
    //         .path("/games/2")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    // }

    // #[tokio::test]
    // async fn put_too_long_content_413() {
    //     let db = mocked_db();
    //     let filter = games_routes(db);

    //     let res = warp::test::request()
    //         .method("PUT")
    //         .header("content-length", 1024 * 36)
    //         .path("/games/2")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    // }

    // #[tokio::test]
    // async fn delete_wrong_id_404() {
    //     let db = mocked_db();
    //     let filter = games_routes(db);

    //     let res = warp::test::request()
    //         .method("DELETE")
    //         .path("/games/42")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::NOT_FOUND);
    // }

    // #[tokio::test]
    // async fn delete_game_204() {
    //     let db = mocked_db();
    //     let filter = games_routes(db.clone());

    //     let res = warp::test::request()
    //         .method("DELETE")
    //         .path("/games/1")
    //         .reply(&filter)
    //         .await;

    //     assert_eq!(res.status(), StatusCode::NO_CONTENT);
    //     assert_eq!(db.lock().await.len(), 1);
    // }
}
