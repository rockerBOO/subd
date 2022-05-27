#![allow(unused_variables)]
#![allow(dead_code)]

// TODO:
// - We could add twitter usernames
//      - See if ppl are following.
//      - Only allow chatters who retweeted my last tweet, everyone else gets timed out
// - Channel points / Channel Redemptions

// - Theme song:
//      - Add a sound
//          - Download the sound locally
//          - Associated sound w/ user_id
//      - Approve/Reject a sound

use std::env;

use anyhow::Result;
use either::Either;
use futures::SinkExt;
use irc::client::prelude::*;
use obws::requests::SceneItemProperties;
use obws::requests::SourceFilterVisibility;
use obws::Client as OBSClient;
use reqwest::Client as ReqwestClient;
use subd_yew::YewTwitchMessage;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use twitch_api2::helix::HelixClient;
use twitch_api2::twitch_oauth2::{AccessToken, UserToken};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

#[derive(Debug, Clone)]
enum Event {
    TwitchChatMessage(PrivmsgMessage),
    TwitchSubscription,
    GithubSponsorshipEvent,
}

const CONNECT_OBS: bool = false;
const CONNECT_CHAT: bool = true;
const CONNECT_HELIX: bool = false;

async fn handle_twitch_msg(
    tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let mut conn = subd_db::get_handle().await;

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(msg) => msg,
            _ => continue,
        };

        let twitch_login = &msg.sender.login;
        println!("Message: {:?}", msg.message_text);

        subd_db::create_twitch_user_chat(&mut conn, &msg.sender.id, &msg.sender.login).await?;
        subd_db::save_twitch_message(&mut conn, &msg.sender.id, &msg.message_text).await?;

        // let user_id = subd_db::get_user_from_twitch_user(&mut conn, twitch_login).await?;
        // let user = subd_db::get_user(&mut conn, &user_id).await?;
        //
        // let count = subd_db::get_message_count_from_today(&mut conn, &user_id).await?;
        // println!("{} messages today", count);

        /*
        if msg.contents.starts_with(":show doggo") && can_control_dog_cam {
            client.send_privmsg("#teej_dv", format!("@{} -> sets doggo", msg.user.login))?;

            // TODO: Start a timer to set it back?
        }

        if msg.contents.starts_with(":show space") {
            if can_control_dog_cam {
                client.send_privmsg("#teej_dv", format!("🚀🚀 @{} 🚀🚀", msg.user.login))?;
            } else {
                client.send_privmsg("#teej_dv", "📻 Houston, we have a problem")?;
            }
        }

        if msg.contents.starts_with(":hide space") {
            if can_control_dog_cam {
                client.send_privmsg(
                    "#teej_dv",
                    format!("'... Landing rocketship' @{}", msg.user.login),
                )?;
            } else {
                client.send_privmsg("#teej_dv", "📻 Houston, we have a problem")?;
            }
        }
        */
    }

    // if msg.contents.starts_with("!set_github") {
    //     subd_db::set_github_user_for_user(
    //         &mut conn,
    //         &user_id,
    //         msg.contents.replace("!set_github", "").trim(),
    //     )
    //     .await
    //     .unwrap_or_else(|e| println!("Nice try, didn't work: {:?}", e))
    // }

    // println!("Saved: {:?}\n", msg);
    // println!(
    //     "We got a message {} {}, from {:?}",
    //     chan, msg, message.prefix
    // );
}

async fn handle_twitch_chat(
    tx: broadcast::Sender<Event>,
    _: broadcast::Receiver<Event>,
) -> Result<()> {
    let conn = subd_db::get_handle().await;
    println!("handle_twitch_chat: got conn");

    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    // let mut client = IRCClient::from_config(Config {
    //     nickname: Some("teej_dv".to_owned()),
    //     server: Some("irc.twitch.tv".to_owned()),
    //     port: Some(6667),
    //     channels: vec!["#teej_dv".to_owned()],
    //     use_tls: Some(false),
    //     password: Some(env::var("CHAT_OAUTH").expect("$CHAT_OAUTH must be set")),
    //     ..Config::default()
    // })

    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);
    client.join("teej_dv".to_owned()).unwrap();

    println!("handle_twitch_chat: waiting for msgs...");
    while let Some(message) = incoming_messages.recv().await {
        // println!("handle_twitch_chat: got a message {:?}", message);

        match message {
            ServerMessage::Privmsg(private) => {
                tx.send(Event::TwitchChatMessage(private))?;
            }
            _ => {} // Command::PRIVMSG(chan, rawmsg) => {
                    //     tx.send(Event::TwitchChatMessage(message.into()))?;
                    // }
                    // _ => {}
        }
    }

    Ok(())
}

async fn yew_inner_loop(stream: TcpStream, mut rx: broadcast::Receiver<Event>) -> Result<()> {
    println!("GOT A NEW CONNECTION: {:?}", stream);

    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    // TODO: Better to split stream so that you can read and write at the same time
    // let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    // read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
    //     .forward(write)
    //     .await
    //     .expect("Failed to forward messages")

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(msg) => msg,
            _ => continue,
        };

        // .send(tungstenite::Message::Text(&msg).unwrap())
        // println!("Got a new message, trying to send... {}", msg.contents);
        // let color = msg
        //     .name_color
        //     .unwrap_or(twitch_irc::message::RGBColor { r: 0, g: 0, b: 0 });

        let color = msg
            .clone()
            .name_color
            .unwrap_or(twitch_irc::message::RGBColor { r: 0, g: 0, b: 0 });
        let color_str = format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b);
        println!("COLOR: {}", color_str);

        ws_stream
            .send(tungstenite::Message::Text(serde_json::to_string(&msg)?))
            .await?;
        //
        // println!("... send message");

        // yew::start_app::<App>();

        // stream.write(msg.contents).await
        // println!("Event: {:?}", event);
    }

    // Ok(())
}

async fn handle_yew(tx: broadcast::Sender<Event>, _: broadcast::Receiver<Event>) -> Result<()> {
    // let ws = tungstenite::connect("ws://127.0.0.1:9001").expect("To be able to open ws");
    let ws = TcpListener::bind("192.168.4.97:9001").await?;
    // let (mut socket, response) = connect(request)

    while let Ok((stream, _)) = ws.accept().await {
        let rx_clone = tx.subscribe();

        tokio::spawn(async move {
            yew_inner_loop(stream, rx_clone)
                .await
                .expect("handling inner loop OK")
        });
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut channels = vec![];
    let (tx, _) = broadcast::channel::<Event>(256);

    let chat_tx = tx.clone();
    let chat_rx = tx.subscribe();

    channels.push(tokio::spawn(async move {
        handle_twitch_chat(chat_tx, chat_rx)
            .await
            .expect("handling twitch chat to work?")
    }));

    let (msg_tx, msg_rx) = (tx.clone(), tx.subscribe());
    channels.push(tokio::spawn(async move {
        handle_twitch_msg(msg_tx, msg_rx)
            .await
            .expect("handling twitch msg to work")
    }));

    let (yew_tx, yew_rx) = (tx.clone(), tx.subscribe());
    channels.push(tokio::spawn(async move {
        handle_yew(yew_tx, yew_rx)
            .await
            .expect("handling yew to work")
    }));

    // handle_commands(...)

    // tokio::spawn(async move {
    //     assert_eq!(rx1.recv().await.unwrap(), 10);
    //     assert_eq!(rx1.recv().await.unwrap(), 20);
    // });

    if CONNECT_OBS {
        // Connect to the OBS instance through obs-websocket.
        let obs_client = OBSClient::connect("192.168.4.22", 4444).await?;

        // Get and print out version information of OBS and obs-websocket.
        let version = obs_client.general().get_version().await?;
        println!("OBS Connected: {:#?}", version.version);

        // Can ignore the following, they were just things that I had working before
        // that I didn't want to forget about later.
        obs_client.scenes().set_current_scene("PC - Dog").await?;
        obs_client
            .sources()
            .set_source_filter_visibility(SourceFilterVisibility {
                source_name: "PC - Elgato",
                filter_name: "SpaceFilter",
                filter_enabled: true,
            })
            .await?;

        obs_client
            .sources()
            .set_source_filter_visibility(SourceFilterVisibility {
                source_name: "PC - Elgato",
                filter_name: "SpaceFilter",
                filter_enabled: false,
            })
            .await?;

        let mut to_set = SceneItemProperties::default();
        // to_set.scene_name = Some("PC");
        to_set.item = Either::Left("PC - Elgato");
        to_set.visible = Some(false);
        obs_client
            .scene_items()
            .set_scene_item_properties(to_set)
            .await?;
    }

    if CONNECT_HELIX {
        // Get  twitch Helix client (don't need for now)
        let helix: HelixClient<ReqwestClient> = HelixClient::default();
        let token = UserToken::from_existing(
            &helix,
            AccessToken::new(
                env::var("TWITCH_OAUTH")
                    .expect("$TWITCH_OAUTH must be set")
                    .replace("oauth:", "")
                    .to_string(),
            ),
            None, // Refresh Token
            None, // Client Secret
        )
        .await?;
    }

    for c in channels {
        // Wait for all the channels to be done
        c.await?;
    }

    Ok(())
}
