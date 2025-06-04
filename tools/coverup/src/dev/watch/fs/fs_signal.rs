use crate::coverdrop_service::CoverDropService;

#[derive(Clone, Debug)]
pub enum FsSignal {
    Dirty(CoverDropService),
}
