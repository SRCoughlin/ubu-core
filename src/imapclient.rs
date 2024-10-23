#![allow(deprecated)]
#![allow(dead_code)]
#![allow(unreachable_code)]

// use imap::ClientBuilder;

pub fn fetch_inbox_top() -> imap::error::Result<Option<String>> {
    let builder = imap::ClientBuilder::new("127.0.0.1", 7660);
    let client = builder.mode(imap::ConnectionMode::Plaintext).connect().unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client
        .login("sean", "T7SRuve0qoyKjEVgHOVdPlHdt")
        // .map_err(|e| e.0).unwrap();
        .unwrap();

    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX").unwrap();

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    let messages = imap_session.fetch("1", "RFC822").unwrap();
    let message = if let Some(m) = messages.iter().next() {
        m
    } else {
        return Ok(None);
    };

    // extract the message's body
    let body = message.body().expect("message did not have a body!");
    let body = std::str::from_utf8(body)
        .expect("message was not valid utf-8")
        .to_string();

    // be nice to the server and log out
    imap_session.logout()?;

    Ok(Some(body))
}
