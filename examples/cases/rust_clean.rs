fn compute(limit: i32) -> Result<i32, &'static str> {
    let mut total = 0;

    for value in 0..limit {
        total += value;
    }

    if total >= 0 {
        Ok(total)
    } else {
        Err("negative total")
    }
}

fn main() {
    match compute(4) {
        Ok(value) => println!("{value}"),
        Err(error) => eprintln!("{error}"),
    }
}
