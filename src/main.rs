use std::collections::HashMap;
use std::{fs, io};
use std::ops::RangeInclusive;
use std::{thread, time};
use std::io::{Read, Write};
use std::sync::mpsc;

use clap::Parser;
use rand::rngs::ThreadRng;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use thirtyfour::prelude::*;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// User ID to start from
    #[clap(long, default_value_t = 364614)]
    start_id: u64,

    /// Login username
    #[clap(short, long)]
    username: String,

    /// Login password
    #[clap(short, long)]
    password: String,

    /// Wait time after error
    #[clap(long, default_value_t = 5000)]
    error_wait_time: u64,

    /// Chromedriver port
    #[clap(long, default_value_t = 9515)]
    chromedriver_port: u32,

    /// Discord webhook to send updates to
    #[clap(long)]
    discord_webhook_url: Option<String>,

    /// Minimum amount of time to wait between tries
    #[clap(long, default_value_t = 1000)]
    wait_min: u64,

    /// Max amount of time to wait between tries
    #[clap(long, default_value_t = 3000)]
    wait_max: u64,

    #[clap(short, long, default_value = "users.json")]
    file: String,
}

struct RandomWait {
    range: RangeInclusive<u64>,
    rng: ThreadRng,
}

impl RandomWait {
    fn new(start: u64, end: u64) -> Self {
        RandomWait {
            range: (start..=end),
            rng: rand::thread_rng(),
        }
    }

    fn gen(&mut self) -> u64 {
        self.rng.gen_range(self.range.clone())
    }
}

async fn wait(message: Option<String>, milliseconds: u64) {
    let mut sp = Spinner::new(Spinners::Dots, match message {
        None => format!("Waiting {milliseconds} milliseconds"),
        Some(m) => format!("{m}, waiting {milliseconds} milliseconds"),
    });

    thread::sleep(time::Duration::from_millis(milliseconds));
    sp.stop_with_newline()
}

#[derive(Serialize, Deserialize)]
struct ProfileApiOk {
    description: Option<String>,
    username: String,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProfileApiBad {
    error: ProfileErr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileErr {
    message: String,
    pretty_message: String,
}

const LOGIN_URL: &str = "https://www.brick-hill.com/login";
const PROFILE_API_URL: &str = "https://api.brick-hill.com/v1/user/profile?id=";
const PROFILE_URL: &str = "https://www.brick-hill.com/user/";

const LOGIN_BUTTON_XPATH: &str = "/html/body/div[1]/div/div/div[2]/form/button";
const FRIEND_BUTTON_XPATH: &str =
    "//div[@class='content text-center bold medium-text relative ellipsis']/div/a[3]";

#[derive(Serialize, Deserialize)]
struct Data {
    users: HashMap<u64, bool>,
}

fn save(data: &Data, file: &str) {
    fs::write(file, serde_json::to_string(data).unwrap()).unwrap();
}

async fn message_webhook(client: &Client, url: &str, content: String) {
    let json = HashMap::from([("content", content)]);

    client
        .post(url)
        .json(&json)
        .send()
        .await
        .expect("Failed to send info to webhook");
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let args = Args::parse();
    let file = args.file;
    let client = Client::new();

    let mut current_id = args.start_id;
    if current_id == 0 {
        current_id = 1
    }

    let mut data: Data = match fs::read_to_string(file.as_str()) {
        Ok(raw_json) => serde_json::from_str(&raw_json).unwrap(),
        Err(_) => Data {
            users: Default::default(),
        },
    };

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new(
        &format!("http://localhost:{}", args.chromedriver_port),
        caps,
    )
    .await
    .expect("I can't connect to chromedriver!");

    // Login to the website
    driver.get(LOGIN_URL).await?;
    driver
        .find_element(By::Id("username"))
        .await?
        .send_keys(&args.username)
        .await?;
    driver
        .find_element(By::Id("password"))
        .await?
        .send_keys(&args.password)
        .await?;

    print!("Press any key when captcha completed");
    io::stdout().flush().unwrap();
    io::stdin().read_exact(&mut [0]).unwrap();

    println!("Thank you! Starting now");
    driver
        .find_element(By::XPath(LOGIN_BUTTON_XPATH))
        .await?
        .click()
        .await?;

    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move ||{
            tx.send(()).expect("Failed to send exit message");
    }).expect("Error setting Ctrl-C handler");

    let mut random_wait = RandomWait::new(args.wait_min, args.wait_max);
    let mut wait_time = random_wait.gen();

    while rx.try_recv().is_err() {
        let api_url: String = format!("{PROFILE_API_URL}{current_id}");
        let url = format!("{PROFILE_URL}{current_id}");

        while let Some(true) = data.users.get(&current_id) {
            println!("Skipping user {current_id}");
            current_id += 1;
        }

        let res = client
            .get(&api_url)
            .send()
            .await
            .expect("failed to query profile api");
        if !res.status().is_success() {
            let response: ProfileApiBad = serde_json::from_str(&res.text().await.unwrap()).unwrap();

            if response.error.message != "Record not found" {
                eprintln!("{response:?}");
            } else {
                wait(Some(response.error.message), args.error_wait_time).await;
            }

            continue;
        }

        let response: ProfileApiOk = serde_json::from_str(&res.text().await.unwrap()).unwrap();

        driver.get(url).await?;

        let friend_button = driver
            .find_element(By::XPath(FRIEND_BUTTON_XPATH))
            .await?;
        match friend_button.text().await?.as_str() {
            "FRIEND" => {
                println!("Friending {current_id} ({})", response.username);
                friend_button.click().await?;
            }
            "CANCEL FRIEND" => {
                println!(
                    "Already sent friend request to {current_id} ({})!",
                    response.username
                );
                wait_time = 0;
            }
            "REMOVE FRIEND" => {
                println!("Already friends with {current_id} ({})!", response.username);
                wait_time = 0;
            }
            _ => {
                println!("This case should never happen");
                current_id += 1;
                continue;
            }
        }

        data.users.insert(current_id, true);

        // Sends info to webhook
        let status_update_format = format!("Username: {}  ID: {current_id}", response.username);
        println!("{status_update_format}");
        if let Some(url) = &args.discord_webhook_url {
            message_webhook(&client, url, status_update_format).await
        }

        wait(None, wait_time).await;

        current_id += 1;
        wait_time = random_wait.gen();
    }

    println!("Exiting, have a good day");
    driver.close().await.expect("Failed to close chrome");
    save(&data, &file);

    Ok(())
}
