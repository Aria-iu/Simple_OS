const FD_STDOUT: usize = 1;

use crate::batch::USER_STACK;
const USER_STACK_SIZE: usize = 4096;
const APP_SIZE_LIMIT: usize = 0x20000;
const APP_BASE_ADDRESS: usize = 0x80400000;

pub fn sys_write(fd: usize,buf: *const u8,len: usize) -> isize{
    match fd {
        FD_STDOUT => {
            // unsafe {println!("#{:#x} {:#x} #", buf as usize , USER_STACK.get_sp() - USER_STACK_SIZE);}
            // 打印数据在用户栈上或者在数据段内
            if (((buf as usize)  >= USER_STACK.get_sp() - USER_STACK_SIZE) && ((buf as usize) + len <= USER_STACK.get_sp())) 
            || (((buf as usize) + len <= APP_SIZE_LIMIT + APP_BASE_ADDRESS) && ((buf as usize) >= APP_BASE_ADDRESS)){
                let slice = unsafe { core::slice::from_raw_parts(buf, len) };
                let str = core::str::from_utf8(slice).unwrap();
                print!("{}", str);
                len as isize
            }else{
                -1 as isize
            }

        },
        _ =>{
            -1 as isize
        }
    }
}