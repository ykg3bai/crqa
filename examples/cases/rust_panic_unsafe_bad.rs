#![allow(dead_code)]

unsafe fn write_value(ptr: *mut i32) {
    unsafe {
        *ptr = 7;
    }
}

fn run(value: Option<i32>) -> i32 {
    let value = value.unwrap();
    let narrowed = value as u8;
    dbg!(narrowed);

    if value > 0 {
        while value > 0 {
            if value > 0 {
                if value > 0 {
                    if value > 0 {
                        panic!("too deep");
                    }
                }
            }
        }
    }

    value
}

fn main() {
    println!("{}", run(Some(1)));
}
