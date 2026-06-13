use std::cell::RefCell;
use std::rc::Rc;

fn build_state() {
    let shared: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let name = String::from("crqa");
    let _copy = name.clone();
    let _owned = name.to_owned();
    let _raw: u32 = unsafe { std::mem::transmute([0u8; 4]) };

    dbg!(&shared);
}

fn main() {
    build_state();
}
