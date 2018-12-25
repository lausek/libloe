pub mod buffer;

pub mod input
{
    pub enum CursorMove
    {
        Absolute(i64, i64),
        Relative(i64, i64),
        EndOfRow(i64),
        CurrentRow(i64),
    }
}

pub mod plugin
{
    pub type NameCallback = unsafe extern "C" fn() -> &'static str;
    pub type CommandsCallback = unsafe extern "C" fn() -> Vec<String>;
    pub type DispatchCallback =
        unsafe extern "C" fn(&mut crate::buffer::Buffer, &str) -> Result<(), String>;
    pub type UnloadCallback = unsafe extern "C" fn();
}
