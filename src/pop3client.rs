#![allow(deprecated)]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]

// use std::vec;
use std::error::Error;
// use mailparse::parse_mail;
// use mailparse::MailHeaderMap;
// use rust_pop3_client::Pop3Connection;
// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use rs_pop3::POP3Stream;
use rs_pop3::POP3Result::{POP3Stat, POP3List, POP3Message, POP3Err};
// use rs_pop3::POP3Result::{POP3Message, POP3Err};
// use email_format::Email;
// use email_format::rfc5322::Parsable;
use mail_parser::*;
// use rand::Rng;
use crate::datamodel::data::*;
use chrono::{Utc, DateTime, FixedOffset, Duration};
use chrono::TimeZone;

const MAX_MESSAGES: u32 = 100;

fn convert_date_time(d: &mail_parser::DateTime) -> chrono::DateTime<Utc> {
    let dt = Utc.with_ymd_and_hms(d.year.into(), d.month.into(), d.day.into(), d.hour.into(), d.minute.into(), d.second.into()).unwrap();
    if d.tz_before_gmt {
        dt + Duration::hours(d.tz_hour.into()) + Duration::minutes(d.tz_minute.into())
    } else {
        dt - Duration::hours(d.tz_hour.into()) - Duration::minutes(d.tz_minute.into())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EmailProcess {
    Leave,
    Delete,
}

pub fn get_email<F>(mut f: F) -> Result<u32, Box<dyn Error>> where F: FnMut(&EmailNotification) -> EmailProcess {
	let host = "127.0.0.1";
    // let host = "localhost";
    let port = 7660;
    let user = "sean";
    let password = "T7SRuve0qoyKjEVgHOVdPlHdt";

	let mut pop_socket = match POP3Stream::connect((host, port), None, host) {
        Ok(s) => s,
        Err(e) => panic!("{}", e)
    };

    let res = pop_socket.login(user, password);
    match res {
        POP3Err => println!("Err logging in"),
        _ => (),
    }

    let stat = pop_socket.stat();
    match stat {
    	POP3Stat {num_email,
				  mailbox_size} => {
                    println!("num_email: {},  mailbox_size:{}", num_email, mailbox_size);
                },
		_ => println!("Err for stat"),
    }

    let mut notification_count = 0;

    let list_all = pop_socket.list(None);
    match list_all {
        POP3List {emails_metadata} => {
            for email in emails_metadata.iter() {
                // println!("message_id: {},  message_size: {}", email.message_id, email.message_size);

                let message = pop_socket.retr(email.message_id);
                match message {
                    POP3Message{raw} => {
                        let text = raw.join("");
                        let message = MessageParser::default().parse(text.as_bytes()).unwrap();
                        // println!("{}\t{} <{}>\t{:.50}\t{:.50}\t{:.40}",
                        //     message.date().unwrap().to_rfc3339(),
                        //     message.from().unwrap().first().unwrap().name().unwrap(),
                        //     message.from().unwrap().first().unwrap().address().unwrap(),
                        //     message.subject().unwrap(),
                        //     message.message_id().unwrap(),
                        //     message.body_html(0).unwrap());

                        if message.attachment_count() > 0 {
                            todo!();
                        }

                        let email_notification = EmailNotification {
                            date: convert_date_time(message.date().unwrap()),
                            message_id: message.message_id().unwrap().into(),
                            from: format!("{} <{}>", message.from().unwrap().first().unwrap().name().unwrap(), message.from().unwrap().first().unwrap().address().unwrap()),
                            subject: message.subject().unwrap().to_string(),
                            body_type: NotificationBodyType::HTML,
                            body: message.body_html(0).unwrap().into(),
                            text: text,
                        };

                        let email_process = f(&email_notification);
                        match email_process {
                            EmailProcess::Leave => {},
                            EmailProcess::Delete => {
                                pop_socket.dele(email.message_id);
                            },
                        }

                        notification_count += 1;
                        println!("completed: {}", notification_count);
                    },
                    _ => println!("Error for message: {:?}", message),
                }

                if notification_count >= MAX_MESSAGES {
                    break;
                }
            }
        },
        _ => println!("Err for list_all"),
    }

    pop_socket.quit();

    Ok(notification_count)
}
