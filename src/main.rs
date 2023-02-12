use std::{time::Duration, sync::Mutex, fs::File, io::Read};

use anyhow::Result;
use log::info;
use serde::Deserialize;
use simplelog::{ConfigBuilder, CombinedLogger, WriteLogger, LevelFilter};
use tokio::spawn;

mod async_rumqttc;

#[derive(Debug, Clone, Deserialize)]
pub struct PeerConfig {
    ip: String,
    port: u16,
    thread: u8,
    conn_per_thread: u16,
    user_name_prefix: String,
    connection_timeout: u64,
    send_message_delay: u64,
    reconnect_delay: u64,
    keep_alive: u64,
    log_type: u8,
}

static mut TOTAL_SUB:Mutex<i32> = Mutex::new(0);
static mut TOTAL_MSG:Mutex<i32> = Mutex::new(0);
static mut TOTAL_SEND:Mutex<i32> = Mutex::new(0);

//ulimit -HSn 65536

fn add_total_send(){
    let mut total = unsafe{ TOTAL_SEND.lock().unwrap() };
    *total += 1;
}

fn add_total_sub(){
    let mut total = unsafe{ TOTAL_SUB.lock().unwrap() };
    *total += 1;
}

fn add_total_msg(){
    let mut total = unsafe{ TOTAL_MSG.lock().unwrap() };
    *total += 1;
}

fn remove_total_sub(){
    let mut total = unsafe{ TOTAL_SUB.lock().unwrap() };
    *total -= 1;
}

fn _init_env_logger(){
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
}

fn _init_simple_log() -> Result<()>{
    let mut builder = ConfigBuilder::new();
    let _ = builder.set_time_offset_to_local();

    CombinedLogger::init(
        vec![
            // TermLogger::new(LevelFilter::Error, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            // TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            // TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),

            // WriteLogger::new(LevelFilter::Error, Config::default(), File::create("app_err.log").unwrap()),
            // WriteLogger::new(LevelFilter::Warn, Config::default(), File::create("app_warn.log").unwrap()),
            WriteLogger::new(LevelFilter::Info,
                builder
                .build()
                , File::create("app.log")?),
        ]
    )?;

    Ok(())
}

fn main() -> Result<()>{

    let mut config_file = File::open("config.toml")?;
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str)?;
    let config: PeerConfig = toml::from_str(&config_str)?;

    if config.log_type == 1{
        _init_env_logger();
    }else{
        _init_simple_log()?;
    }

    info!("开始...");
    for i in 0..config.thread as u16{
        let config_clone = config.clone();
        let x = i*config_clone.conn_per_thread;
        std::thread::spawn(move ||{
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(test_thread(config_clone.clone(), x, config.conn_per_thread));
            info!("线程{i}结束")
        });
    }
    loop{
        let total = unsafe{ TOTAL_MSG.try_lock() };
        let total_sub = unsafe{ TOTAL_SUB.try_lock() };
        let total_send = unsafe{ TOTAL_SEND.try_lock() };
        if let (Ok(total), Ok(sub), Ok(send)) = (total, total_sub, total_send){
            info!("当前订阅总数:{sub}, 已接收消息:{total}, 已发送消息:{send}");
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

async fn test_thread(config: PeerConfig, start: u16, size: u16){
    for i in start..start+size{
        //随机sleep
        // let delay = rand::random::<u8>();
        tokio::time::sleep(Duration::from_millis(i as u64/2)).await;
        let config_clone = config.clone();
        let username = format!("{}{i}", config_clone.user_name_prefix);
        spawn(async move {
            // let res = async_mqtt::subscribe(username).await;
            let uname = username.clone();
            loop{
                // info!("开始链接:{uname}...");
                let _res = async_rumqttc::subscribe(config_clone.clone(), &uname).await;
                remove_total_sub();
                tokio::time::sleep(Duration::from_secs(config.reconnect_delay)).await;
            }
        });
    }
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
