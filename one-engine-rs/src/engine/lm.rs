use serde_json::Value;
use async_openai::{Client, types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, Role}};

#[derive(Clone, Debug)]
pub struct PlanCand { pub steps: Vec<String>, pub risk: f32, pub uncertainty: f32 }

#[async_trait::async_trait]
pub trait Lm: Send + Sync {
    async fn plan(&self, goal: &str, inputs: &Value) -> anyhow::Result<PlanCand>;
    async fn critic_pre(&self, _step: &str, _inputs: &Value) -> anyhow::Result<f32> { Ok(0.1) }
}

/// Stub LM: proposes multiple steps for SWE goals
pub struct StubLm;

#[async_trait::async_trait]
impl Lm for StubLm {
    async fn plan(&self, goal: &str, _inputs: &Value) -> anyhow::Result<PlanCand> {
        let steps = if goal.starts_with("swe.") {
            vec![
                "echo 'Running SWE task...'".to_string(),
                "ls -la".to_string(),
                "pwd".to_string(),
                "echo 'SWE task completed'".to_string()
            ]
        } else if goal.starts_with("chat.") {
            vec!["echo 'Hello! I am the one-engine assistant.'".to_string()]
        } else {
            vec!["demo.write-file".to_string()]
        };
        
        Ok(PlanCand { steps, risk: 0.1, uncertainty: 0.1 })
    }
}

pub struct OpenAiLm { pub client: Client<async_openai::config::OpenAIConfig> }

impl OpenAiLm {
    pub fn new() -> anyhow::Result<Self> {
        if std::env::var("OPENAI_API_KEY").is_err() {
            anyhow::bail!("OPENAI_API_KEY not set");
        }
        Ok(Self { client: Client::new() })
    }
}

#[async_trait::async_trait]
impl Lm for OpenAiLm {
    async fn plan(&self, goal: &str, inputs: &Value) -> anyhow::Result<PlanCand> {
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: "You are a planner. Output JSON with {\"steps\":[\"cmd1\", \"cmd2\"]} for shell commands to achieve the goal.".to_string(),
                    name: None,
                    role: Role::System,
                }
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: format!("Goal: {}\nInputs: {}", goal, inputs).into(),
                    name: None,
                    role: Role::User,
                }
            )
        ];
        
        let req = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(messages)
            .build()?;
        let resp = self.client.chat().create(req).await?;
        let content = resp.choices[0].message.content.as_ref().unwrap();
        
        // Try to parse JSON, fallback to simple commands
        if let Ok(v) = serde_json::from_str::<Value>(content) {
            if let Some(steps_array) = v["steps"].as_array() {
                let steps = steps_array.iter()
                    .map(|s| s.as_str().unwrap_or("echo 'invalid'").to_string())
                    .collect();
                return Ok(PlanCand { steps, risk: 0.1, uncertainty: 0.1 });
            }
        }
        
        // Fallback: treat response as single command
        Ok(PlanCand { 
            steps: vec![format!("echo '{}'", content.replace("'", "''"))], 
            risk: 0.1, 
            uncertainty: 0.1 
        })
    }
}
