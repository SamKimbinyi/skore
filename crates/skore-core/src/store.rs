pub trait Store: Send + Sync {
    fn get() -> ();
    fn set() -> ();
    fn delete() -> ();
    fn flush() -> ();
}