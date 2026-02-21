use imap_next::imap_types::{
    command::CommandBody,
    mailbox::Mailbox,
    response::{Data, StatusBody, StatusKind},
    status::{StatusDataItem, StatusDataItemName},
};

use super::TaskError;
use crate::tasks::Task;

#[derive(Clone, Debug)]
pub struct StatusTask {
    mailbox: Mailbox<'static>,
    item_names: Vec<StatusDataItemName>,
    output: Vec<StatusDataItem>,
}

impl StatusTask {
    pub fn new(mailbox: Mailbox<'static>, item_names: Vec<StatusDataItemName>) -> Self {
        Self {
            mailbox,
            item_names,
            output: Vec::new(),
        }
    }
}

impl Task for StatusTask {
    type Output = Result<Vec<StatusDataItem>, TaskError>;

    fn command_body(&self) -> CommandBody<'static> {
        CommandBody::Status {
            mailbox: self.mailbox.clone(),
            item_names: self.item_names.clone().into(),
        }
    }

    fn process_data(&mut self, data: Data<'static>) -> Option<Data<'static>> {
        match data {
            Data::Status { items, .. } => {
                self.output = items.to_vec();
                None
            }
            data => Some(data),
        }
    }

    fn process_tagged(self, status_body: StatusBody<'static>) -> Self::Output {
        match status_body.kind {
            StatusKind::Ok => Ok(self.output),
            StatusKind::No => Err(TaskError::UnexpectedNoResponse(status_body)),
            StatusKind::Bad => Err(TaskError::UnexpectedBadResponse(status_body)),
        }
    }
}
