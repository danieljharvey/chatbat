use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::fmt;
use tokio::io::AsyncBufReadExt;
use tokio::io::{ AsyncWriteExt};
use tokio_stream::StreamExt;

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
    // By default, it will connect to localhost:11434
    let ollama = Ollama::default();

    let model = "llama3.2:latest";

    println!("Let's have a completely nice chat. What's up?");

    let mut context = vec![];

    loop {
        let input = read_line().await.unwrap();

        let response = prompt(&ollama, &model, &input, &context).await.unwrap();

        println!("\n{}\n", response);

        context.push(Chat { input, response });
    }
}

fn create_prompt(new_input: &str, _context: &Vec<Chat>) -> String {
    
    let json_schema = include_str!("schema.json");

     let prompt = format!("Hello! You have a job creating workflows. They should be as simple as possible. Please format your response like this: {{ message: string, workflow: JSON }}. The JSON should use this JSONSchema:\n\n{json_schema}\n\n");

    format!("{prompt}\nHere are the requirements:\n\n{new_input}")
}

async fn prompt(
    ollama: &Ollama,
    model: &str,
    prompt: &str,
    context: &Vec<Chat>,
) -> Result<String, MyError> {
    let total_prompt = create_prompt(prompt, context);

    let mut stream = ollama
        .generate_stream(GenerationRequest::new(model.to_string(), total_prompt))
        .await
        .unwrap();

    let mut buffer = Vec::new();
    while let Some(res) = stream.next().await {
        let responses = res.unwrap();
            print!(".");
        for resp in responses {
            buffer.write_all(resp.response.as_bytes()).await.unwrap();
        }
    }
    String::from_utf8(buffer).map_err(|_| MyError::Other)
}
