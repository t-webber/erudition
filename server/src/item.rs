use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Item {
    MultipleChoice {
        question: String,
        answers: Vec<String>,
    },
}
