use clap::Parser;
use color_eyre::Result;
use reqwest::Client;
use rustyline::DefaultEditor;
use serde_json::{Value, json};
use std::io::{self, Write};
use std::time::Duration;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "chat-cli")]
#[command(about = "A CLI chat interface for the Gateway Service")]
struct Cli {
    /// Gateway service URL
    #[arg(short, long, default_value = "http://localhost:9000")]
    url: String,
}

struct ChatSession {
    client: Client,
    gateway_url: String,
    conversation_id: Option<Uuid>,
}

impl ChatSession {
    fn new(gateway_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            gateway_url,
            conversation_id: None,
        }
    }

    async fn send_message(&mut self, message: &str) -> Result<String> {
        let mut payload = json!({
            "message": message
        });

        if let Some(conv_id) = self.conversation_id {
            payload["conversation_id"] = json!(conv_id);
        }

        let response = self
            .client
            .post(format!("{}/api/chat", self.gateway_url))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            let error_msg = if error_text.is_empty() {
                format!("Gateway service returned status {}", status)
            } else {
                format!("Gateway service error {}: {}", status, error_text)
            };
            return Err(color_eyre::eyre::eyre!(error_msg));
        }

        let response_json: Value = response.json().await?;

        // Extract conversation_id for future messages
        if let Some(conv_id_str) = response_json
            .get("conversation_id")
            .and_then(|v| v.as_str())
            && let Ok(conv_id) = Uuid::parse_str(conv_id_str) {
                self.conversation_id = Some(conv_id);
            }

        // Extract the message from the response
        let message = response_json
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message in response");

        Ok(message.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    println!("🤖 Gateway Chat CLI");
    println!("Connected to: {}", cli.url);
    println!("Type 'exit' or press Ctrl+C to quit");
    println!("───────────────────────────────────────");

    let mut session = ChatSession::new(cli.url);
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline("💬 You: ");

        match readline {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                if input.eq_ignore_ascii_case("exit") {
                    println!("👋 Goodbye!");
                    break;
                }

                rl.add_history_entry(input)?;

                print!("🔄 Processing...");
                io::stdout().flush()?;

                match session.send_message(input).await {
                    Ok(response) => {
                        print!("\r🤖 Assistant: ");
                        println!("{}\n", response);
                    }
                    Err(e) => {
                        print!("\r❌ Error: ");
                        let error_msg = e.to_string();
                        if error_msg.contains("Connection refused") {
                            println!(
                                "Cannot connect to gateway service at {}. Is it running?\n",
                                session.gateway_url
                            );
                        } else if error_msg.contains("timeout") {
                            println!("Request timed out. The gateway service may be overloaded.\n");
                        } else {
                            println!("{}\n", error_msg);
                        }
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("👋 Goodbye!");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("👋 Goodbye!");
                break;
            }
            Err(err) => {
                println!("❌ Error reading input: {}", err);
                break;
            }
        }
    }

    Ok(())
}
