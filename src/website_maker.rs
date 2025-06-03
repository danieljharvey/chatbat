use super::shared;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;
use std::{fmt::Debug, fs::File};

pub async fn main() {
    println!(
        "Excellent. I am a tool for making new websites for the rock band Olympians. Please describe the vibe of the website you would like please?"
    );

    let mut messages = vec![];

    loop {
        let input = shared::read_line().await.unwrap();

        let prompt = create_prompt(&input);

        let response: PlanResponse = shared::prompt(&prompt, &mut messages).await.unwrap();

        let PlanResponse::NewWebsite(Html { html }) = response;

        println!("{}", html);

        let mut path = env::current_dir().unwrap();
        path.push("website.html");

        let mut output = File::create(&path).unwrap();
        write!(output, "{}", html).unwrap();
        println!("Written to {:?}", path);
    }
}

fn create_prompt(new_input: &str) -> String {
    let original_html = include_str!("./static/website.html");

    let prompt = format!(
        "Hello! We're redesigning a website. The original website is a single html file that looks like the following:\n\n{}\n\n. I would like you to regenerate a new version of it as a single html page with all styles inlined. It should contains all the text and content from the previous site. Make sure to make it very exciting.",
        original_html
    );

    format!("{prompt}\nHere are the requirements:\n\n{new_input}")
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
enum PlanResponse {
    NewWebsite(Html),
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
struct Html {
    html: String,
}
