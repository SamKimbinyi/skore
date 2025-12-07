use skore_core::Store;
use std::sync::Arc;

pub struct Server<S: Store> {
    store: Arc<S>,
    addr: String,
}
