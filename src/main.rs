#![feature(box_into_inner)]

use std::sync::Arc;

use bot::Bot;
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod bot;
mod cobalt;
mod commands;
mod craiyon;
mod mathjs;
mod poligon;
mod translate;
mod urbandictionary;
mod utils;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().unwrap();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("craiyon_bot", LevelFilter::Debug)
        .init()
        .unwrap();

    let mut bot = Bot::new();
    bot.add_command("start", Arc::new(commands::start::Start));
    bot.add_command("generate", Arc::new(commands::generate::Generate));
    bot.add_command("translate", Arc::new(commands::translate::Translate));
    bot.add_command("badtranslate", Arc::new(commands::badtranslate::BadTranslate));
    bot.add_command("urbandictionary", Arc::new(commands::urbandictionary::UrbanDictionary));
    bot.add_command("cobalt_download", Arc::new(commands::cobalt_download::CobaltDownload));
    bot.add_command("charinfo", Arc::new(commands::charinfo::CharInfo));
    bot.add_command("startit_joke", Arc::new(commands::startit_joke::StartItJoke));

    bot.run().await;
}
