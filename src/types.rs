use std::result::Result as StdResult;
use uuid::Uuid;

use crate::errors::Error;

pub type PlayerId = Uuid;
pub type TeamId = Uuid;
pub type Result<T, E = Error> = StdResult<T, E>;
