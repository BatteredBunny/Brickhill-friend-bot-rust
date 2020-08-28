use thirtyfour::prelude::*;
use tokio;
use std::io;
use std::fs::OpenOptions;
use std::fs;
use std::{thread, time};
use std::io::Write;
use serde_json::{Value};

#[tokio::main]
async fn main() -> WebDriverResult<()> {

    let starting_id: u64 = 364614;
    let wait_time: u64 = 5000;
    let error_wait_time: u64 = 10000;

    let mut current_id = loop {
        println!("Enter starting id (default {:?}):", starting_id);
        let mut current_id = String::new();

        io::stdin()
        .read_line(&mut current_id)
        .expect("Failed to read line");

        if current_id.trim().is_empty() == true {
            break starting_id;
        } else if current_id.trim().parse::<u64>().is_ok() == true{
            let current_id: u64 = current_id.trim().parse().expect("t");
            break current_id;
        }
    };
    println!("You chose {}", current_id);

    println!("Please enter your username:");
    let mut username = String::new();
    io::stdin()
    .read_line(&mut username)
    .expect("Failed to read line");

    println!("Please enter your password:");
    let mut password = String::new();
    io::stdin()
    .read_line(&mut password)
    .expect("Failed to read line");

    //username txt creation teen siin
    let _file = OpenOptions::new().write(true)
                             .create_new(true)
                             .open("users.txt");

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", &caps).await.expect("Please open chromedriver at port 4444");

    driver.get("https://www.brick-hill.com/login").await?;

    let password_field = driver.find_element(By::Id("password")).await?;
    let username_field = driver.find_element(By::Id("username")).await?;

    password_field.send_keys(&password).await?;
    username_field.send_keys(&username).await?;

    loop {
        if current_id == 0 {
			current_id += 1;
        }
        let api_url = format!("https://api.brick-hill.com/v1/user/profile?id={}",current_id);
        let url = format!("https://www.brick-hill.com/user/{}", current_id);

        // info format [username:id]
        let info_format = format!("[{}:{}]", &username, &current_id.to_string());

        let contents = fs::read_to_string("users.txt")
        .expect("Something went wrong reading the file");

        //has to read whole file, search in the string.
        if contents.contains(&info_format) {
            println!("Skipping user {}", &current_id);
            current_id += 1;
            continue;
        } 
        let json_user_info = reqwest::get(&api_url)
            .await?
            .text()
            .await?;
        let user_info: Value = serde_json::from_str(&json_user_info)?;
        if user_info["error"] == "Record not found" {
            loop {
                thread::sleep(time::Duration::from_millis(error_wait_time));
                if user_info["error"] != "Record not found" {
                    break;
                }
            }
        }

        driver.get(url).await?; 

        let friend_button = driver.find_element(By::XPath("//a[@class='button small inline blue']")).await?;
        friend_button.click().await?;

        // here adds line to file
        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("users.txt")
        .unwrap();

        if let Err(e) = writeln!(file, "{}", &info_format) {
            eprintln!("Couldn't write to file: {}", e);
        }

        let message_format = format!("Username: {}  ID: {}", user_info["username"], &current_id);
        println!("{}", message_format);

        //make it give 5-10s random wait time
        println!("Waiting {} milliseconds", &wait_time);
        thread::sleep(time::Duration::from_millis(wait_time));
        current_id += 1;
    }
}