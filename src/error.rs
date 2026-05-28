pub fn user_error(message: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!(message.into())
}
