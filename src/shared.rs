use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fmt;
use std::fmt::Debug;
use tokio::io::AsyncBufReadExt;

#[derive(Debug, PartialEq)]
pub enum MyError {
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

pub async fn read_line() -> Result<String, MyError> {
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut buffer = Vec::new();

    let _fut = reader.read_until(b'\n', &mut buffer).await;
    String::from_utf8(buffer).map_err(|_| MyError::Other)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OllamaResponse {
    model: String,
    created_at: String,
    message: Message,
}

#[derive(Debug, PartialEq)]
pub struct ResponseWith<A> {
    pub response: A,
    pub similarity: usize,
}

pub async fn multi_prompt<A>(
    input: &str,
    messages: &mut Vec<Message>,
) -> Result<ResponseWith<A>, MyError>
where
    A: for<'a> serde::Deserialize<'a> + serde::Serialize + JsonSchema + Debug + PartialEq,
{
    let mut messages_copy = messages.clone();
    let resp_a = prompt::<A>(input, messages).await?;
    let resp_b = prompt::<A>(input, &mut messages_copy).await?;

    let resp_a_string = serde_json::to_string(&resp_a).unwrap().to_ascii_uppercase();
    let resp_b_string = serde_json::to_string(&resp_b).unwrap().to_ascii_uppercase();

    // Levenshtein distance
    let distance = distance::levenshtein(resp_a_string.as_str(), resp_b_string.as_str());

    let max_len = max(resp_a_string.len(), resp_b_string.len());

    let similarity = distance * 100 / max_len;

    Ok(ResponseWith {
        response: resp_a,
        similarity,
    })
}

async fn prompt<A>(prompt: &str, messages: &mut Vec<Message>) -> Result<A, MyError>
where
    A: for<'a> serde::Deserialize<'a> + JsonSchema,
{
    let url = "http://localhost:11435/api/chat";
    messages.push(Message {
        role: "user".to_string(),
        content: prompt.to_string(),
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
