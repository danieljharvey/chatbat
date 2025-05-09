use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::io::AsyncBufReadExt;

#[derive(Debug)]
enum MyError {
    Other,
}

impl std::error::Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Other => write!(f, "Some other error occured!"),
        }
    }
}

async fn read_line() -> Result<String, MyError> {
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut buffer = Vec::new();

    let _fut = reader.read_until(b'\n', &mut buffer).await;
    String::from_utf8(buffer).map_err(|_| MyError::Other)
}

#[tokio::main]
async fn main() {
    println!("Please run: `OLLAMA_HOST=127.0.0.1:11435 ollama serve`");
    println!("Let's have a completely nice chat. What's up?");

    let mut messages = vec![];

    loop {
        let input = read_line().await.unwrap();

        let country = prompt(&input, &mut messages).await.unwrap();

        println!("\n{:?}\n", country);
    }
}

fn create_prompt(new_input: &str) -> String {
    let prompt = "Hello! Please help me remember countries.".to_string();

    format!("{prompt}\nHere are the requirements:\n\n{new_input}")
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OllamaResponse {
    model: String,
    created_at: String,
    message: Message,
}

#[derive(Debug, Deserialize, Serialize)]
struct Country {
    name: String,
    capital: String,
    languages: Vec<String>,
}

async fn prompt(prompt: &str, messages: &mut Vec<Message>) -> Result<Country, MyError> {
    let total_prompt = create_prompt(prompt);

    let url = "http://localhost:11435/api/chat";
    messages.push(Message {
        role: "user".to_string(),
        content: total_prompt,
    });

    let json = serde_json::json!(
    {
      "model": "llama3.2",
      "messages": messages,
      "stream": false,
      "format": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string"
          },
          "capital": {
            "type": "string"
          },
          "languages": {
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        },
        "required": [
          "name",
          "capital",
          "languages"
        ]
      }
    }
            );

    let client = reqwest::Client::new();
    let res = client.post(url).json(&json).send().await.unwrap();

    let json_response: OllamaResponse = res.json().await.unwrap();

    let inner_json = serde_json::from_str(&json_response.message.content).unwrap();

    // keep response for content
    messages.push(json_response.message);

    Ok(inner_json)
}
