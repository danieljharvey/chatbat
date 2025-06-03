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

        match response.response_a {
            PlanResponse::Plan(plan) => {
                println!("Accuracy {}%", response.compare_score);
                println!("{}", serde_json::to_string_pretty(&plan).unwrap());
            }
            PlanResponse::FollowUpQuestion(question) => {
                println!("{}", question.question);
            }
            PlanResponse::RequestMyLocation => {
                println!("Where are you currently?");
            }
        }
    }
}

fn create_prompt(new_input: &str) -> String {
    let prompt = r#"
        Hello! 

        I need you to help me break down some tasks into steps. 

        If anything is unclear, please return a `FollowUpQuestion`.
        If you'd like to know my current location, return a `RequestMyLocation`, but only once.

        If you have enough information, return a `Plan`.

        Things to ensure before returning a plan:
        - I know where I am travelling, and how I will get there  
        - If I need to take anything with me, and how I will transport those things
        - Who is coming with me.

        Keep the titles short and snappy, and include a list of items I will require with each step. 
        "#.to_string();

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
    RequestMyLocation,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
struct FollowUpQuestion {
    question: String,
}
