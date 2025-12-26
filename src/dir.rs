use std::path::PathBuf;

pub(crate) fn default_clip_dir() -> PathBuf {
    let documents_dir = dirs_next::document_dir()
        .or_else(dirs_next::home_dir)
        .expect("Unable to determine user directory");

    documents_dir
        .join("danmarkowal")
        .join("rolling-sampler-clone")
        .join("clips")
}
