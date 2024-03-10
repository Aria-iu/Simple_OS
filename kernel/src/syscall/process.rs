use crate::batch::run_next_app;

pub fn sys_exit(exit_code: i32) -> isize{
    println!("[kernel] Application exited with code {}", exit_code);
    // this is in batch system we can do
    run_next_app()
}