use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod shared;
mod task_planner;
mod website_maker;

fn create_prompt(new_input: &str) -> String {
    let prompt = "Hello! I need you to decide which program we're going to run.".to_string();

    format!("{prompt}\nHere are the requirements:\n\n{new_input}")
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
enum ProgramType {
    TaskPlanner,
    WebsiteMaker,
}

#[tokio::main]
async fn main() {
    println!("Please run: `OLLAMA_HOST=127.0.0.1:11435 ollama serve`");
    println!("How can I help you today?");

    let mut messages = vec![];

    let input = shared::read_line().await.unwrap();

    let prompt = create_prompt(&input);

    let response: ProgramType = shared::prompt(&prompt, &mut messages).await.unwrap();

    match response {
        ProgramType::TaskPlanner => task_planner::main().await,
        ProgramType::WebsiteMaker => website_maker::main().await,
    }
}
