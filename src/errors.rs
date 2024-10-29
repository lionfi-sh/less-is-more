use thruster::errors::ThrusterError;

use crate::app::Ctx;

pub enum Error {
    GenericError(Ctx, String, #[allow(dead_code)] serde_json::Value),
}

impl Into<ThrusterError<Ctx>> for Error {
    fn into(self) -> ThrusterError<Ctx> {
        match self {
            Error::GenericError(context, message, _) => ThrusterError {
                context,
                message,
                cause: None,
            },
        }
    }
}
