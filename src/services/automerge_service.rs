use crate::services::connection_service::ConnectionCommand;
use anyhow::{anyhow, bail, Error, Result};
use automerge::sync::{Message as AutomergeSyncMessage, State as SyncState, State, SyncDoc};
use automerge::{AutoCommit, ChangeHash, ObjId, ReadDoc};
use chrono::{DateTime, Local};
use dioxus::hooks::use_coroutine_handle;
use dioxus::prelude::{Coroutine, GlobalSignal, ReadableExt, Signal};
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use std::fmt::Display;

#[derive(Clone, PartialEq)]
pub struct AutomergeDocumentFile {
    pub file_name: String,
    pub content: String,
}

pub struct MessageDetails {
    heads: String,
    last_sync: String,
    need: String,
    version: String,
}

impl MessageDetails {
    fn from_message(message: &AutomergeSyncMessage) -> Result<Self> {
        let last_sync: Vec<ChangeHash> = message
            .have
            .iter()
            .flat_map(|h| h.last_sync.clone())
            .collect();
        Ok(Self {
            last_sync: serde_json::to_string_pretty(&last_sync)?,
            heads: serde_json::to_string_pretty(&message.heads)?,
            need: serde_json::to_string_pretty(&message.need)?,
            version: format!("{:?}", message.version),
        })
    }
}

impl Display for MessageDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "last_sync: {}\nheads: {}\nneed: {}\nversion: {}",
            self.last_sync, self.heads, self.need, self.version
        )
    }
}

pub enum AutomergeEvent {
    AppliedSyncMessage {
        date_time: DateTime<Local>,
        details: MessageDetails,
    },
    CreatedSyncMessage {
        date_time: DateTime<Local>,
        details: MessageDetails,
    },
    Error {
        date_time: DateTime<Local>,
        error: Error,
    },
}

impl Display for AutomergeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutomergeEvent::AppliedSyncMessage { date_time, details } => {
                write!(f, "{date_time}: applied sync message:\n{details}")
            }
            AutomergeEvent::CreatedSyncMessage { date_time, details } => {
                write!(f, "{date_time}: created sync message:\n{details}")
            }
            AutomergeEvent::Error { date_time, error } => {
                write!(f, "{date_time}: automerge error {error}")
            }
        }
    }
}

pub static AUTOMERGE_EVENTS: GlobalSignal<Vec<AutomergeEvent>> = Signal::global(Vec::new);

pub static FILES: GlobalSignal<Vec<String>> = Signal::global(Vec::new);
pub static SELECTED_FILE: GlobalSignal<Option<AutomergeDocumentFile>> = Signal::global(|| None);

async fn apply_message(
    doc: &mut AutoCommit,
    state: &mut State,
    message: AutomergeSyncMessage,
) -> Result<Vec<ChangeHash>> {
    let mut new_doc = doc.fork();
    new_doc.sync().receive_sync_message(state, message)?;
    Ok(doc.merge(&mut new_doc)?)
}

fn object_id_by_name(doc: &AutoCommit, parent: ObjId, name: &str) -> Result<ObjId> {
    if let Some(object_id) = doc.get(parent, name)?.map(|entry| entry.1) {
        return Ok(object_id);
    }

    bail!("no object '{name}' found!")
}

fn files_object(doc: &AutoCommit) -> Result<ObjId> {
    object_id_by_name(doc, automerge::ROOT, "files")
}

fn files(doc: &AutoCommit) -> Result<Vec<String>> {
    Ok(doc.keys(files_object(doc)?).collect())
}

fn file_content(doc: &AutoCommit, file_name: &str) -> Result<String> {
    let object_id = object_id_by_name(doc, files_object(doc)?, file_name)?;
    Ok(doc.text(object_id)?)
}

fn select_file(doc: &AutoCommit, file_name: &str) -> Result<()> {
    *SELECTED_FILE.write() = Some(AutomergeDocumentFile {
        file_name: file_name.to_owned(),
        content: file_content(doc, file_name)?,
    });
    Ok(())
}

pub enum AutomergeCommand {
    ApplyMessage { message: AutomergeSyncMessage },
    SelectFile { file_name: String },
    StartSync,
}

async fn handle_automerge_command(
    doc: &mut AutoCommit,
    state: &mut State,
    command: AutomergeCommand,
    connection_service: Coroutine<ConnectionCommand>,
) -> Result<()> {
    match command {
        AutomergeCommand::ApplyMessage { message } => {
            let details = MessageDetails::from_message(&message)?;
            apply_message(doc, state, message).await?;
            AUTOMERGE_EVENTS
                .write()
                .push(AutomergeEvent::AppliedSyncMessage {
                    date_time: Local::now(),
                    details,
                });

            *FILES.write() = files(doc)?;

            if let Some(previous_selected_file) = SELECTED_FILE.read().as_ref() {
                let file_name = &previous_selected_file.file_name;
                if FILES.read().contains(file_name) {
                    select_file(doc, file_name)?;
                } else {
                    *SELECTED_FILE.write() = None;
                }
            }
        }
        AutomergeCommand::SelectFile { ref file_name } => {
            select_file(doc, file_name)?;
        }
        AutomergeCommand::StartSync => {
            while let Some(message) = doc.sync().generate_sync_message(state) {
                let details = MessageDetails::from_message(&message)?;
                AUTOMERGE_EVENTS
                    .write()
                    .push(AutomergeEvent::CreatedSyncMessage {
                        date_time: Local::now(),
                        details,
                    });
                connection_service.send(ConnectionCommand::SendMessage { message });
            }
        }
    }
    Ok(())
}

fn handle_error(error: Error) {
    AUTOMERGE_EVENTS.write().push(AutomergeEvent::Error {
        date_time: Local::now(),
        error,
    });
}

pub async fn start_automerge_service(mut commands_rx: UnboundedReceiver<AutomergeCommand>) {
    let connection_service = use_coroutine_handle::<ConnectionCommand>();

    // TODO: load content from local storage?
    // see https://github.com/ethersync/ethersync/blob/v0.7.0/daemon/src/document.rs#L37
    let initial_doc = [
        133, 111, 74, 131, 61, 157, 231, 85, 0, 118, 1, 16, 120, 107, 104, 47, 215, 9, 76, 32, 132,
        136, 60, 124, 152, 120, 144, 182, 1, 143, 164, 31, 13, 102, 61, 139, 125, 246, 189, 135,
        97, 16, 167, 63, 30, 215, 249, 60, 227, 113, 111, 61, 55, 138, 234, 94, 30, 142, 166, 78,
        250, 6, 1, 2, 3, 2, 19, 2, 35, 2, 64, 2, 86, 2, 7, 21, 14, 33, 2, 35, 2, 52, 1, 66, 2, 86,
        2, 128, 1, 2, 127, 0, 127, 1, 127, 2, 127, 0, 127, 0, 127, 7, 126, 5, 102, 105, 108, 101,
        115, 6, 115, 116, 97, 116, 101, 115, 2, 0, 2, 1, 2, 2, 0, 2, 0, 2, 0, 0,
    ];
    let maybe_doc = AutoCommit::load(&initial_doc);
    if let Err(error) = maybe_doc {
        handle_error(anyhow!(error));
        return;
    }

    let mut doc = maybe_doc.unwrap();
    let mut state = SyncState::default();

    while let Some(command) = commands_rx.next().await {
        if let Err(error) =
            handle_automerge_command(&mut doc, &mut state, command, connection_service).await
        {
            handle_error(error);
        }
    }
}
