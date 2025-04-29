use super::{Factor, FactorOutputError, Mode, Question, QuestionProps};

pub struct PasswordFactor {
    mode: Mode,
}

impl PasswordFactor {
    pub fn new(block: &[u8; 64], mode: Mode) -> Self {
        Self { mode }
    }

    pub fn next_question(&mut self) -> Option<Question> {
        let (q, chan) = Question::new("".into(), QuestionProps::Password { allow_toggle: true });
        Some(q)
    }

    pub fn result(self) -> Result<[u8; 64], FactorOutputError> {
        Ok([0u8; 64])
    }
}

impl Factor for PasswordFactor {}

// pub struct PasswordFactorQuestions {
//     mode: Mode,
// }

// impl PasswordFactorQuestions {
//     pub fn next_question(&mut self) -> Option<Question> {
//         let (q, chan) = Question::new("".into(), QuestionProps::Password { allow_toggle: true });
//         Some(q)
//     }
// }
