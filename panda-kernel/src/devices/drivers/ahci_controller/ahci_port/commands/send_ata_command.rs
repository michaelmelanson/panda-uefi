#[derive(Clone, Debug)]
pub enum SendATACommandReply {
    Success,
}

impl Default for SendATACommandReply {
    fn default() -> Self {
        SendATACommandReply::Success
    }
}
