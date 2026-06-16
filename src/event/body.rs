use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommitEvent {
    pub repo: String,
    pub branch: String,
    pub sha: String,
    pub short_sha: String,
    pub summary: String,
    pub repo_path: Option<String>,
    pub worktree_path: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranchChangedEvent {
    pub repo: String,
    pub old_branch: String,
    pub new_branch: String,
    pub repo_path: Option<String>,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubIssueEvent {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubPullRequestEvent {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub branch: String,
    pub base_branch: Option<String>,
    pub sha: Option<String>,
    pub short_sha: Option<String>,
    pub author: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubCheckEvent {
    pub owner: String,
    pub repo: String,
    pub workflow: String,
    pub status: String,
    pub branch: String,
    pub sha: Option<String>,
    pub short_sha: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubReleaseEvent {
    pub owner: String,
    pub repo: String,
    pub tag: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesGatewayEvent {
    pub provider: Option<String>,
    pub source: Option<String>,
    pub platform: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesSessionEvent {
    pub status: String,
    pub session_id: Option<String>,
    pub platform: Option<String>,
    pub project: Option<String>,
    pub message_chars: Option<u64>,
    pub response_chars: Option<u64>,
    pub has_message: Option<bool>,
    pub has_response: Option<bool>,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesAgentEvent {
    pub status: String,
    pub agent_name: Option<String>,
    pub session_id: Option<String>,
    pub platform: Option<String>,
    pub project: Option<String>,
    pub step_name: Option<String>,
    pub message_chars: Option<u64>,
    pub response_chars: Option<u64>,
    pub has_message: Option<bool>,
    pub has_response: Option<bool>,
    pub elapsed_secs: Option<u64>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomEvent {
    pub kind: String,
    pub message: String,
    pub payload: Option<Value>,
}
