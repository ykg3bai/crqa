use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub targets: Vec<Target>,
    pub yaml: bool,
    pub quiet: bool,
    pub help: bool,
    pub output: Option<PathBuf>,
    pub ai_review: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    File(PathBuf),
    Directory(PathBuf),
}

pub fn print_usage(program: &str) {
    println!(
        "Usage: {program} -f FILE [options]\n\
         \n\
         Options:\n\
           -f FILE                Check one C/C++/Rust source file. Can be repeated.\n\
           -d DIR                 Recursively check C/C++/Rust files under DIR.\n\
           -y, --yaml             Print YAML output.\n\
           -o, --output FILE      Write report to FILE instead of stdout.\n\
           -q, --quiet            Show only error-level diagnostics.\n\
           --aireview             Add DeepSeek AI review score. Requires DEEPSEEK_API_KEY.\n\
           -h, --help             Show this help."
    );
}

pub fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut targets = Vec::new();
    let mut yaml = false;
    let mut quiet = false;
    let mut help = false;
    let mut output = None;
    let mut ai_review = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-f" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err("missing path after -f".to_string());
                };
                targets.push(Target::File(PathBuf::from(path)));
            }
            "-d" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err("missing path after -d".to_string());
                };
                targets.push(Target::Directory(PathBuf::from(path)));
            }
            "-y" | "--yaml" => yaml = true,
            "-o" | "--output" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err(format!("missing path after {}", args[index - 1]));
                };
                output = Some(PathBuf::from(path));
            }
            option if option.starts_with("--output=") => {
                output = Some(PathBuf::from(&option["--output=".len()..]));
            }
            "-q" | "--quiet" => quiet = true,
            "-h" | "--help" => help = true,
            "--aireview" => ai_review = true,
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
        index += 1;
    }

    Ok(Config {
        targets,
        yaml,
        quiet,
        help,
        output,
        ai_review,
    })
}
