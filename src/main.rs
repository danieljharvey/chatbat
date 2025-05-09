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

struct Chat {
    input: String,
    response: String,
}

#[tokio::main]
async fn main() {
    let model = "llama3.2:latest";
    let json_schema = include_str!("schema.json");

    println!("Let's have a completely nice chat. What's up?");

    loop {
        let input = read_line().await.unwrap();

        let response = prompt(&model, &input).await.unwrap();

        println!("\n{}\n", response);
    }
}

fn create_prompt(new_input: &str) -> String {
    let prompt = "Hello! You have a job creating fake users.".to_string();

    format!("{prompt}\nHere are the requirements:\n\n{new_input}")
}

async fn prompt(model: &str, prompt: &str) -> Result<String, MyError> {
    let total_prompt = create_prompt(prompt);

    let url = "http://localhost:11434/api/chat";
    let json = serde_json::json!(
    {
      "model": "llama3.1",
      "messages": [{"role": "user", "content": total_prompt}],
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

    dbg!(&res);

    Ok("".to_string())
}
