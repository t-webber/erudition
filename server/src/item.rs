use serde::{Deserialize, Serialize};

/// Item store and returned by the server
#[derive(Serialize, Deserialize, Debug)]
pub enum Item {
    /// Multiple choice question
    MultipleChoice {
        /// Possible answers, the first is always the
        /// correct answer
        answers: Vec<String>,
        /// Question
        question: String,
    },
}
