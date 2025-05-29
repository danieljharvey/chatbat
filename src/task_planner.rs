use super::shared;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub async fn main() {
    println!("Excellent. I am a task planning tool. What would you like me to help you plan?");

    let mut messages = vec![];

    loop {
        let input = shared::read_line().await.unwrap();

        let prompt = create_prompt(&input);

        let response: shared::ResponseWith<PlanResponse> =
            shared::multi_prompt(&prompt, &mut messages).await.unwrap();

        println!(
            "{}",
            serde_json::to_string_pretty(&response.response).unwrap()
        );
        println!("Accuracy {}%", response.similarity);
    }
}

fn create_prompt(new_input: &str) -> String {
    let prompt = "Hello! I need you to help me break down some tasks into steps. Keep the titles short and snappy, and include a list of items I will require with each step. Please request further clarifications if anything is unclear by returning a FollowUpQuestion. I expect multiple tasks, please ask for more details if you do not yet have any.".to_string();

    format!("{prompt}\nHere are the requirements:\n\n{new_input}")
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
struct Task {
    title: String,
    instructions: String,
    items: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
struct Plan {
    tasks: Vec<Task>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
enum PlanResponse {
    Plan(Plan),
    FollowUpQuestion(FollowUpQuestion),
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
struct FollowUpQuestion {
    question: String,
}
