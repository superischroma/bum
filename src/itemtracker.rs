use chrono::{DateTime, Local};
//use chrono::Utc;
use std::{thread, io::Write, time::{SystemTime, Duration}, collections::HashMap};

use json::JsonValue;

use crate::front;

const PERIOD: u64 = 1;
const DAY: u64 = 8.64e7 as u64;

fn req_ah(page: u32) -> Option<JsonValue>
{
    let url = format!("https://api.hypixel.net/skyblock/auctions?page={}", page);
    let result = reqwest::blocking::get(url);
    if result.is_err()
    {
        ()
    }
    let response = result.unwrap();
    if !response.status().is_success()
    {
        ()
    }
    let text_result = response.text();
    if text_result.is_err()
    {
        ()
    }
    let text = text_result.unwrap();
    Option::Some(json::parse(text.as_str()).unwrap())
}

fn wait(secs: u64)
{
    let t = SystemTime::now();
    let dur_secs = Duration::from_secs(secs);
    while SystemTime::now() < t + dur_secs {
        //let next_update = t + dur_secs;
        //let time_diff = next_update - SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        //let dt_time_diff = DateTime::<Utc>::from(time_diff);
        //let ts_time_diff = dt_time_diff.format("%H:%M:%S").to_string();
        //let dt_next_update = DateTime::<Local>::from(next_update);
        //let ts_next_update = dt_next_update.format("%r").to_string();
        //println!("{} left until next update ({})", ts_time_diff, ts_next_update);
        thread::sleep(Duration::from_secs(10));
    }
    thread::sleep(Duration::from_secs(5));
}

fn send_notification(message: &str)
{
    front::send_notification("Item Tracker", message)
}

fn run()
{
    thread::sleep(Duration::from_secs(20)); // wait a lil
    let mut items: HashMap<&str, Vec<u64>> = HashMap::from([
        ("Ender Artifact", vec![]),
        ("Bedrock", vec![])
    ]);

    'process_loop: loop
    {
        let test_opt = req_ah(0);
        if test_opt.is_none()
        {
            send_notification("Hypixel API down");
            wait(300);
            continue;
        }
        let test = test_opt.unwrap();
        send_notification("Pulling item data...");
        let pages = test["totalPages"].as_u32().unwrap();
        for i in 0..pages
        {
            let data_opt = req_ah(i);
            if data_opt.is_none()
            {
                send_notification("Hypixel API down");
                wait(300);
                continue 'process_loop;
            }
            let data = data_opt.unwrap();
            let auctions = &data["auctions"];
            for auction in auctions.members()
            {
                if !auction["bin"].as_bool().unwrap()
                {
                    continue;
                }
                if auction["start"].as_u64().unwrap() < SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64 - (DAY * PERIOD)
                {
                    continue;
                }
                let item_name = auction["item_name"].as_str().unwrap();
                if items.contains_key(item_name)
                {
                    items.get_mut(item_name).unwrap().push(auction["starting_bid"].as_u64().unwrap());
                }
            }
        }
        for entry in items.iter_mut()
        {
            let item_name = entry.0;
            let bids = entry.1;
            if bids.len() == 0
            {
                send_notification(format!("No {}s found currently", item_name).as_str());
            }
            else
            {
                bids.sort();
                let q1 = bids[(bids.len() as f64 * 0.25) as usize];
                let q3 = bids[(bids.len() as f64 * 0.75) as usize];
                let iqr = (q3 - q1) as f64;
                let filtered: Vec<u64> = bids.clone().into_iter().filter(|&i| i > q1 - (iqr * 1.5) as u64 && i < q3 + (iqr * 1.5) as u64).collect();
                let fin = if filtered.len() != 0 { filtered } else { bids.to_vec() };
                let sum: u64 = fin.iter().sum();
                let coins = (sum as f64 / fin.len() as f64).round().to_string().as_bytes().rchunks(3).rev().map(std::str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap().join(",");
                send_notification(format!("{}: {} coins", item_name, coins).as_str());
            }
            bids.clear();
            thread::sleep(Duration::from_secs(5));
        }
        let next_update = SystemTime::now() + Duration::from_secs(3600);
        let dt_next_update = DateTime::<Local>::from(next_update);
        let ts_next_update = dt_next_update.format("%r").to_string();
        send_notification(format!("Next update at {}", ts_next_update).as_str());
        wait(3600);
    }
}

pub fn start_process()
{
    thread::spawn(run);
}

pub fn edit()
{
    loop
    {
        println!("-- Edit Item Tracker --");
        println!("[1] Return");
        print!(">> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input).expect("failed to read input from user");
        input.retain(|c| c != '\r' && c != '\n');
        match input.as_str() {
            "1" => {
                break;
            },
            _ => {
                println!("unexpected input: {}", input)
            }
        };
    }
}