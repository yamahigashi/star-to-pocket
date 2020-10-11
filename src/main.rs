#[macro_use]
extern crate clap;
use clap::{App, Arg};

extern crate config;
// use std::collections::HashMap;

use std::io;
use std::error::Error;
use std::time::Instant;

use std::fs::OpenOptions;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use shellexpand;

extern crate pocket;
// extern crate github_starchaser;
// use github_starchaser::{Star, github_star};
use github_rs::client::{Github, Executor};
use serde_json::Value;


extern crate serde_json;

use url::Url;

// use serde::Serialize;

// use urlencoding::encode;

use pocket::{add::PocketAddRequest, auth::PocketAuthentication, Pocket};


#[tokio::main]
async fn post_to_pocket(pocket: &Pocket, star: &Value) -> Result<(), Box<dyn Error>> {

    let url = Url::parse(star["html_url"].as_str().unwrap())?;
    let title = match star["full_name"].as_str() {
        Some(o) => o,
        _ => url.as_str()
    };

    let mut tags = match star["language"].as_str() {
        Some(o) => vec!["github", o],
        _ => vec!["github"],
    };

    match star["topics"].as_str() {
        Some(ref o) => {
            tags.push(o);
        },
        _ => {},
    };

    let mut file = OpenOptions::new().read(true).append(true).create(true).open("saved_to_pocket.txt").unwrap();
    if is_saved(&file, &url) {
        println!("already saved {}", title);
        return Ok(());
    }

    let item = pocket
        .add(
            &PocketAddRequest::new(&url)
                .title(title)
                .tags(&tags),
        )
        .await?;

    println!("item: {:?}", item);
    match write!(&mut file, "{}\n", url.as_str()) {
        Err(e) => println!("write to file falied, {}", e),
        _ => (),
    };

    Ok(())

}


fn is_saved(file: &File, url: &Url) -> bool {

    let buffered = BufReader::new(file);

    for line in buffered.lines() {
        match line {
            Err(e) => println!("{}", e),
            Ok(o) => {
                // if o == o {
                if *url.as_str() == o {
                    return true;
                }
            },
        };
    }

    false

}


fn test_pocket(_username: &str, github_token: &str, consumer_key: &str, access_token: &str) {

    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!());

    let pocket = Pocket::new(consumer_key, access_token);

    let _ = app.get_matches();

    for body in gather_star(github_token).unwrap().as_array() {
        for star in body.into_iter().rev() {
            // println!("{:#?}", repo["description"]);
            match post_to_pocket(&pocket, star) {
                Err(e) => println!("{}", e),
                _ => {},
            };
        }
    }

}


fn gather_star(github_token: &str) -> Option<Value> {
    let client = Github::new(github_token).unwrap();
    let me = client.get()
                    .users()
                    .username("yamahigashi")
                    .starred()
                    .execute::<Value>();

    match me {
        Ok((_headers, _status, json)) => {
            // println!("{:#?}", headers);
            // println!("{}", status);
            json
        },
        Err(e) => {
            println!("{}", e);
            None
        }
    }
}


#[tokio::main]
async fn auth(consumer_key: &str) -> Result<(), Box<dyn Error>> {
    println!("consumer key: {}", consumer_key);
    let auth = PocketAuthentication::new(&consumer_key, "rustapi:finishauth");
    let state = Some(format!("{:?}", Instant::now()));
    let code = auth.request(state.as_deref()).await?;
    let url = auth.authorize_url(&code);
    println!(
        "Follow auth URL to provide access and press enter when finished: {}",
        url
    );
    let _ = io::stdin().read_line(&mut String::new());
    let user = auth.authorize(&code, state.as_deref()).await?;
    println!("username: {}", user.username);
    println!("access token: {:?}", user.access_token);
    Ok(())
}


fn main() {
    let config_path = shellexpand::tilde("~/githubstar_to_pocket");

    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name(&config_path)).unwrap()

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::with_prefix("STAR_TO_POCKET")).unwrap();

    test_pocket(settings.get::<String>("username").unwrap().as_str(),
                settings.get::<String>("github_access_token").unwrap().as_str(),
                settings.get::<String>("pocket_consumer_key").unwrap().as_str(),
                settings.get::<String>("pocket_access_token").unwrap().as_str());
    // auth(pocket_consumer_key).unwrap();
}
