use std::env;
use reqwest::header::COOKIE;
use std::io::Read;
use select::document::Document;
use select::predicate::{Name, Class, Attr};

use futures::StreamExt;
use telegram_bot::*;

use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

struct MyData {
    melli_card: String,
    token: String,
}

async fn add_melli_card(api: Api, message: Message) -> Result<(), Error> {
    api.send(message.chat.text("plz insert your melli card")).await?;

    Ok(())
}

async fn add_token(api: Api, message: Message) -> Result<(), Error> {
    api.send(message.chat.text("plz insert your token")).await?;

    Ok(())
}

async fn get_sallary(api: Api, message: Message, my_data: &mut MyData) -> Result<(), Error> {

    if !my_data.melli_card.is_empty() {

        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let thread_tx = tx.clone();

        let origin_url = "https://vebko.ir/Automation/SalaryPersonal.aspx?req=".to_string() + &my_data.melli_card;
        let token = my_data.token.to_string();
        println!("{}", origin_url);
        println!("{}", token);

        let thread = thread::spawn(move || {
            let mut body = String::new();
            let client = reqwest::blocking::Client::new();

            let mut res = client.get(&*origin_url)
                .header(COOKIE, token)
                .send()
                .unwrap();

            // println!("Status for {}: {}", origin_url, res.status());

            res.read_to_string(&mut body).unwrap();
            println!("HTML: {}", &body);

            let mut i = 0;
            let mut salary = String::new();

            Document::from(body.as_str())
                .find(Attr("id", "PanelSalaryPersonal"))
                .next()
                .unwrap()
                .find(Class("E"))
                .for_each(|x| {
                    if i == 136 {
                        println!("{:?}", x.text());
                        salary = x.text();
                    }
                    i += 1;
                });

            thread_tx.send(salary.to_string()).unwrap();
        });

        thread.join();

        api.send(message.chat.text("your salary = ".to_string() + &rx.recv().unwrap())).await?;
    }

    Ok(())
}

async fn bot(api: Api, message: Message, status: &mut String, my_data: &mut MyData) -> Result<(), Error> {
    match message.kind {
        MessageKind::Text { ref data, .. } => match data.as_str() {
            "/addMelliCard" => {
                status.clear();
                status.push_str("AddMelliCard");
                add_melli_card(api, message).await?
            }
            "/addToken" => {
                status.clear();
                status.push_str("AddToken");
                add_token(api, message).await?
            }
            "/getSalary" => {
                status.clear();
                status.push_str("");
                get_sallary(api, message, my_data).await?
            }
            _ => {
                if status == "AddMelliCard" {
                    my_data.melli_card = message.text().unwrap();
                } else if status == "AddToken" {
                    my_data.token = message.text().unwrap();
                }
            }
        },
        _ => (),
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);

    let mut status = String::from("AddCard");

    let mut my_data = MyData {
        melli_card: String::new(),
        token: String::new(),
    };

    let mut stream = api.stream();

    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            bot(api.clone(), message, &mut status, &mut my_data).await?;
        }
    }

    Ok(())
}