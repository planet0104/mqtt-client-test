use std::time::{Duration, Instant};
use anyhow::Result;
use log::info;
use rumqttc::v5::{MqttOptions, AsyncClient, mqttbytes::QoS, Event};

use crate::{add_total_sub, add_total_msg, add_total_send, PeerConfig};

pub async fn subscribe(config: PeerConfig, username: &str) -> Result<()> {

    let username = username.to_string();
    let host = config.ip.to_string();
    let port = config.port;
    // info!("创建链接:{host} {port}");

    let mut mqttoptions = MqttOptions::new(&username, host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(config.keep_alive));
    mqttoptions.set_connection_timeout(config.connection_timeout);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    client.subscribe(&username, QoS::AtMostOnce).await?;

    add_total_sub();

    // 延迟10分钟再发送消息
    let username1 = username.to_string();
    let username2 = username1.clone();
    tokio::task::spawn(async move {
        loop{
            tokio::time::sleep(Duration::from_secs(config.send_message_delay)).await;
            if let Err(_err) = client.publish(&username, QoS::AtLeastOnce, false, format!("hello:{username1}").as_bytes().to_vec()).await{
                // error!("{username1}发送失败:{:?}", err);
            }else{
                add_total_send();
            }
        }
    });

    let connect_time = Instant::now();
    

    loop{
        match eventloop.poll().await{
            Ok(notification) =>{
                // info!("Received = {:?}", notification);
                if let Event::Outgoing(msg) = notification{
                    if let rumqttc::Outgoing::Publish(_ms1) = msg{
                        add_total_msg();
                    }
                }
            },
            Err(_err) => {
                // error!("{username}链接断开:{:?}", err);
                break;
            }
        }
        let alive_minute = connect_time.elapsed().as_secs() / 60;
        if alive_minute >= config.reconnect_duration{
            info!("{username2}已上线{alive_minute}分钟，自动断开.");
            break;
        }
    }

    Ok(())
}