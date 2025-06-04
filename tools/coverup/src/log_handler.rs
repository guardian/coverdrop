use tokio::sync::broadcast::Sender;

use crate::{coverdrop_service::CoverDropService, dev::watch::builder::BuilderSignal};

pub enum LogHandler<'a> {
    ForwardLogForBuilder(CoverDropService, &'a mut Sender<BuilderSignal>),
    None,
}
