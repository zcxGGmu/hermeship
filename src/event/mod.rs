pub mod body;
pub mod compat;

pub use body::{
    CustomEvent, GitBranchChangedEvent, GitCommitEvent, HermesAgentEvent, HermesGatewayEvent,
    HermesSessionEvent,
};

use time::OffsetDateTime;
use uuid::Uuid;

use crate::events::MessageFormat;

#[derive(Debug, Clone, PartialEq)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub timestamp: OffsetDateTime,
    pub source: String,
    pub body: EventBody,
    pub metadata: EventMetadata,
}

impl EventEnvelope {
    pub fn canonical_kind(&self) -> &str {
        self.body.canonical_kind()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventBody {
    GitCommit(GitCommitEvent),
    GitBranchChanged(GitBranchChangedEvent),
    HermesGatewayStarted(HermesGatewayEvent),
    HermesSessionStarted(HermesSessionEvent),
    HermesSessionFinished(HermesSessionEvent),
    HermesSessionReset(HermesSessionEvent),
    HermesAgentStarted(HermesAgentEvent),
    HermesAgentStep(HermesAgentEvent),
    HermesAgentFinished(HermesAgentEvent),
    HermesAgentFailed(HermesAgentEvent),
    Custom(CustomEvent),
}

impl EventBody {
    pub fn canonical_kind(&self) -> &str {
        match self {
            Self::GitCommit(_) => "git.commit",
            Self::GitBranchChanged(_) => "git.branch-changed",
            Self::HermesGatewayStarted(_) => "hermes.gateway.started",
            Self::HermesSessionStarted(_) => "hermes.session.started",
            Self::HermesSessionFinished(_) => "hermes.session.finished",
            Self::HermesSessionReset(_) => "hermes.session.reset",
            Self::HermesAgentStarted(_) => "hermes.agent.started",
            Self::HermesAgentStep(_) => "hermes.agent.step",
            Self::HermesAgentFinished(_) => "hermes.agent.finished",
            Self::HermesAgentFailed(_) => "hermes.agent.failed",
            Self::Custom(event) => event.kind.as_str(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventMetadata {
    pub channel_hint: Option<String>,
    pub mention: Option<String>,
    pub format: Option<MessageFormat>,
    pub template: Option<String>,
    pub priority: EventPriority,
    pub tool: Option<String>,
    pub provider: Option<String>,
    pub source: Option<String>,
    pub platform: Option<String>,
    pub user_id: Option<String>,
    pub chat_id: Option<String>,
    pub thread_id: Option<String>,
    pub chat_type: Option<String>,
    pub session_id: Option<String>,
    pub agent_name: Option<String>,
    pub project: Option<String>,
    pub repo_name: Option<String>,
    pub repo_path: Option<String>,
    pub worktree_path: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventPriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}
