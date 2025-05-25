use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use tokio::io::AsyncBufReadExt;

#[derive(Debug, PartialEq)]
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
    println!("Describe a country");

    let mut messages = vec![];

    loop {
        let input = read_line().await.unwrap();

        let response: ResponseWith<Country> = multi_prompt(&input, &mut messages).await.unwrap();

        println!("\n{:?}\n", response.response);
        println!("Accuracy {}%", response.similarity);
    }
}

fn create_prompt(new_input: &str) -> String {
    let prompt = "Hello! Please help me remember countries.".to_string();

    format!("{prompt}\nHere are the requirements:\n\n{new_input}")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
struct Country {
    name: String,
    capital: String,
    languages: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct ResponseWith<A> {
    response: A,
    similarity: usize,
}

async fn multi_prompt<A>(
    input: &str,
    messages: &mut Vec<Message>,
) -> Result<ResponseWith<A>, MyError>
where
    A: for<'a> serde::Deserialize<'a> + serde::Serialize + JsonSchema + Debug + PartialEq,
{
    let mut messages_copy = messages.clone();
    let resp_a = prompt::<A>(input, messages).await?;
    let resp_b = prompt::<A>(input, &mut messages_copy).await?;

    // Levenshtein distance
    let distance = distance::levenshtein(
        serde_json::to_string(&resp_a)
            .unwrap()
            .to_ascii_uppercase()
            .as_str(),
        serde_json::to_string(&resp_b)
            .unwrap()
            .to_ascii_uppercase()
            .as_str(),
    );

    Ok(ResponseWith {
        response: resp_a,
        similarity: 100 - distance,
    })
}

async fn prompt<A>(prompt: &str, messages: &mut Vec<Message>) -> Result<A, MyError>
where
    A: for<'a> serde::Deserialize<'a> + JsonSchema,
{
    let total_prompt = create_prompt(prompt);

    let url = "http://localhost:11435/api/chat";
    messages.push(Message {
        role: "user".to_string(),
        content: total_prompt,
    });

    let schema = schema_for!(A);

    let json = serde_json::json!({
      "model": "llama3.2",
      "messages": messages,
      "stream": false,
      "format": schema
    });

    let client = reqwest::Client::new();
    let res = client.post(url).json(&json).send().await.unwrap();

    let json_response: OllamaResponse = res.json().await.unwrap();

    let inner_json = serde_json::from_str(&json_response.message.content).unwrap();

    // keep response for content
    messages.push(json_response.message);

    Ok(inner_json)
}
