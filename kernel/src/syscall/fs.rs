use crate::task::current_user_token;
use crate::mm::translate_byte_buffer;
const FD_STDOUT: usize = 1;

// use crate::batch::USER_STACK;
const USER_STACK_SIZE: usize = 4096;
const APP_SIZE_LIMIT: usize = 0x20000;
const APP_BASE_ADDRESS: usize = 0x80400000;

pub fn sys_write(fd: usize,buf: *const u8,len: usize) -> isize{
    match fd {
        FD_STDOUT => {
            /* 
            /* now it comes to muti at a time version */
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
            */
            /* 
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
            */
            // 由于内核和应用地址空间的隔离，sys_write 不再能够直接访问位于应用空间中的
            // 数据，而需要手动查页表才能知道那些数据被放置在哪些物理页帧上并进行访问。
            let buffers = translate_byte_buffer(current_user_token(),buf,len);
            for buffer in buffers{
                println!("{}",core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        },
        _ =>{
            -1 as isize
        }
    }
}