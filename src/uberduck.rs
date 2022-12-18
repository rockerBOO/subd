use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use rodio::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::{BufWriter, Write};
use std::{thread, time};
use subd_types::Event;
use subd_types::SourceVisibilityRequest;
use subd_types::StreamCharacterRequest;
use subd_types::TransformOBSTextRequest;
use tokio::sync::broadcast;

pub struct UberDuckHandler {
    pub sink: Sink,
}

#[derive(Serialize, Deserialize, Debug)]
struct UberDuckVoiceResponse {
    uuid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UberDuckFileResponse {
    path: Option<String>,
    started_at: Option<String>,
    failed_at: Option<String>,
    finished_at: Option<String>,
}

// Should they be optional???
pub struct StreamCharacter {
    // text_source: String,
    voice: String,
    source: String,
    username: String,
}

#[async_trait]
impl EventHandler for UberDuckHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UberDuckRequest(msg) => msg,
                _ => continue,
            };

            // We need to check the message
            //
            let ch = msg.message.chars().next().unwrap();
            if ch == '!' {
                continue;
            };

            let stream_character = build_stream_character(&msg.username);

            let (username, secret) = uberduck_creds();

            let client = reqwest::Client::new();
            let res = client
                .post("https://api.uberduck.ai/speak")
                .basic_auth(username.clone(), Some(secret.clone()))
                .json(&[
                    ("speech", msg.voice_text),
                    ("voice", stream_character.voice.clone()),
                ])
                .send()
                .await?
                .json::<UberDuckVoiceResponse>()
                .await?;

            let uuid = match res.uuid {
                Some(x) => x,
                None => continue,
            };

            loop {
                let url = format!(
                    "https://api.uberduck.ai/speak-status?uuid={}",
                    &uuid
                );

                let (username, secret) = uberduck_creds();
                let response = client
                    .get(url)
                    .basic_auth(username, Some(secret))
                    .send()
                    .await?;

                // Show Loading Duck
                let _ = tx.send(Event::SourceVisibilityRequest(
                    SourceVisibilityRequest {
                        scene: "Characters".to_string(),
                        source: "loading_duck".to_string(),
                        enabled: true,
                    },
                ));

                let text = response.text().await?;
                // we need to this to be better
                let file_resp: UberDuckFileResponse =
                    serde_json::from_str(&text)?;
                println!("Uberduck Finished at: {:?}", file_resp.finished_at);

                match file_resp.path {
                    Some(new_url) => {
                        // Hide Loading Duck
                        let _ = tx.send(Event::SourceVisibilityRequest(
                            SourceVisibilityRequest {
                                scene: "Characters".to_string(),
                                source: "loading_duck".to_string(),
                                enabled: false,
                            },
                        ));

                        let text_source =
                            format!("{}-text", stream_character.source);
                        let _ = tx.send(Event::TransformOBSTextRequest(
                            TransformOBSTextRequest {
                                message: msg.message.clone(),
                                text_source,
                            },
                        ));
                        // TODO Should we change this file name
                        // Make it unique
                        let local_path = "./test.wav";
                        let response = client.get(new_url).send().await?;
                        let file = File::create(local_path)?;
                        let mut writer = BufWriter::new(file);
                        writer.write_all(&response.bytes().await?)?;
                        println!("Downloaded File From Uberduck, Playing Soon: {:?}!", local_path);

                        let source = stream_character.source.clone();
                        let _ = tx.send(Event::StreamCharacterRequest(
                            StreamCharacterRequest {
                                source,
                                enabled: true,
                            },
                        ));

                        // Hmm We shouldn't fail here then
                        let file =
                            BufReader::new(File::open(local_path).unwrap());
                        self.sink.append(
                            Decoder::new(BufReader::new(file)).unwrap(),
                        );
                        self.sink.sleep_until_end();

                        // THIS IS HIDING THE PERSON AFTER
                        // We might want to wait a little longer, then hide
                        // we could also kick off a hide event
                        let ten_millis = time::Duration::from_millis(1000);
                        thread::sleep(ten_millis);

                        let source = stream_character.source.clone();
                        let _ = tx.send(Event::StreamCharacterRequest(
                            StreamCharacterRequest {
                                source,
                                enabled: false,
                            },
                        ));
                        break;
                    }
                    None => {
                        // Wait 1 second before seeing if the file is ready.
                        let ten_millis = time::Duration::from_millis(1000);
                        thread::sleep(ten_millis);
                    }
                }
            }
        }
    }
}

fn uberduck_creds() -> (String, String) {
    let username = env::var("UBER_DUCK_KEY")
        .expect("Failed to read UBER_DUCK_KEY environment variable");
    let secret = env::var("UBER_DUCK_SECRET")
        .expect("Failed to read UBER_DUCK_SECRET environment variable");
    (username, secret)
}

// ======================================

// Character Builder
// Then Just use that
fn build_stream_character(username: &str) -> StreamCharacter {
    // Start with username
    //
    // Username picks Voice
    //
    // Voice picks Source and Hotkeys

    // let base_source = "Seal";
    // let base_source = "Birb";
    // let base_source = "Kevin";
    // let base_source = "Crabs";
    // let base_source = "Teej";
    // let base_source = "ArtMatt";

    // ====== //
    // VOICES //
    // ====== //
    let default_voice = "brock-samson";
    // let default_voice = "danny-devito-angry";
    // let default_voice = "goku";
    // let default_voice = "mickey-mouse";
    // let default_voice = "mojo-jojo";
    // let default_voice = "tommy-pickles";

    let voices2: HashMap<&str, &str> = HashMap::from([
        ("beginbot", "mr-krabs-joewhyte"),
        // ("beginbot", "danny-devito-angry"),
        // ("beginbot", "big-gay-al"),
        // ("beginbot", "mojo-jojo"),
        // ("beginbot", "mr-krabs-joewhyte"),
        // ("beginbot", "mojo-jojo"),
        // ("beginbot", "theneedledrop"),
        // ("beginbot", "mojo-jojo"),
        // ("beginbot", "chief-keef"),
        ("beginbotbot", "brock-samson"),
        // ("beginbotbot", "theneedledrop"),
        ("ArtMattDank", "dr-nick"),
        // ("ArtMattDank", "mojo-jojo"),
        ("carlvandergeest", "danny-devito-angry"),
        ("stupac62", "stewie-griffin"),
        ("swenson", "mike-wazowski"),
        ("teej_dv", "mr-krabs-joewhyte"),
        // ("theprimeagen", "big-gay-al"),
    ]);

    let voice = match voices2.get(username) {
        Some(v) => v,
        None => default_voice,
    };

    let character = find_obs_character(voice);

    // let text_source = format!("{}-text", character.source);
    StreamCharacter {
        username: username.to_string(),
        voice: voice.to_string(),

        // Why
        // text_source: character.text_source.to_string(),
        source: character.to_string(),
    }
}

// All 6 Filters
// I think we should try alternative filter triggering instead
// we need to trigger 3 filters each time
// and we can get the names based off a pattern
// This is not the ideal method
fn find_obs_character(voice: &str) -> &str {
    // This makes no sense
    let default_hotkeys = "Seal";

    // We need defaults for the source
    // TODO: We need one of these for each voice
    let mut hotkeys: HashMap<&str, &str> = HashMap::from([
        // ("brock-samson", "Seal"),
        ("brock-samson", "Seal"),
        // ("theneedledrop", "Birb"),
        // ("theneedledrop", "Kevin"),
        // ("theneedledrop", "Seal"),
        ("theneedledrop", "ArtMatt"),
        // ("mojo-jojo", "Birb"),
        // ("mojo-jojo", "Teej"),
        // ("mojo-jojo", "ArtMatt"),
        ("mr-krabs-joewhyte", "Crabs"),
        ("danny-devito-angry", "Kevin"),
    ]);

    match hotkeys.remove(voice) {
        Some(v) => v,
        None => default_hotkeys,
    }
}
