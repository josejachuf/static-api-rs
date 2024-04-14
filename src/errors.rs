use thiserror::Error;
use std::io;
use salvo::prelude::*;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("io: `{0}`")]
    Io(#[from] io::Error),

    #[error("json parsing: `{0}`")]
    JsonParse(#[from] serde_json::Error),

    #[error("error parsing request: `{0}`")]
    ParseError(#[from] salvo::http::ParseError),

    // #[error("Failed to generate random ID")]
    // RandomIdGeneration,

    #[error("Item not found with ID: {0}")]
    ItemNotFound(u64),
}

pub type AppResult<T> = Result<T, AppError>;

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.render(Text::Plain(self.to_string()));
    }
}
