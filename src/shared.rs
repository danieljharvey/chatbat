use futures::future;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
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
    pub response_a: A,
    pub response_b: A,
    pub compare_score: u32,
}

pub async fn multi_prompt<A>(
    input: &str,
    messages: &mut Vec<Message>,
) -> Result<ResponseWith<A>, MyError>
where
    A: for<'a> serde::Deserialize<'a> + serde::Serialize + JsonSchema + Debug + PartialEq,
{
    let mut messages_copy = messages.clone();

    let a = prompt::<A>(input, messages);
    let b = prompt::<A>(input, &mut messages_copy);

    let (resp_a, resp_b) = future::join(a, b).await;
    let resp_a = resp_a?;
    let resp_b = resp_b?;

    let resp_a_string = serde_json::to_string(&resp_a).unwrap().to_ascii_uppercase();
    let resp_b_string = serde_json::to_string(&resp_b).unwrap().to_ascii_uppercase();

    let compare_score = compare_with_llm(&resp_a_string, &resp_b_string).await?;

    Ok(ResponseWith {
        response_a: resp_a,
        response_b: resp_b,
        compare_score,
    })
}

async fn compare_with_llm(a: &str, b: &str) -> Result<u32, MyError> {
    prompt(
        &format!(
            "Please compare {} and {} and describe how similar they are between 1 and 100",
            a, b,
        ),
        &mut vec![],
    )
    .await
}

pub async fn prompt<A>(prompt: &str, messages: &mut Vec<Message>) -> Result<A, MyError>
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
      "model": "qwen2.5-coder:7b", //"deepseek-r1:7b",
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
