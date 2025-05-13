use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{exit, Command},
};

use regex::Regex;

fn print_help() {
    println!(
        "\
fastbash — quick script manager

USAGE:
    fastbash create         # Create a new script interactively
    fastbash edit <script>  # Open script for editing
    fastbash <script> [...] # Run a saved script with optional args
    fastbash ls             # List saved scripts
    fastbash rm <script>    # Delete a saved script
    fastbash help           # Show this help message

NOTES:
    - Scripts are saved in ~/.fastbash/scripts
    - Make sure your scripts start with a shebang line (e.g., #!/bin/bash)
    - Set the EDITOR env variable to control which editor is used
"
    );
}

fn extract_description(path: &PathBuf) -> String {
    let re = Regex::new(r"(?i)^#\s*(description|desc)\s*:\s*(.+)$").unwrap();

    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten().take(5) {
            if let Some(caps) = re.captures(&line) {
                return caps.get(2).map_or("(no description)".to_string(), |m| m.as_str().trim().to_string());
            }
        }
    }
    "(no description)".to_string()
}

fn get_scripts_dir() -> PathBuf {
    let mut dir = dirs::home_dir().expect("Could not determine home directory");
    dir.push(".fastbash/scripts");
    fs::create_dir_all(&dir).expect("Failed to create script directory");
    dir
}

fn open_in_editor(path: &PathBuf) {
    let editor_string = env::var("EDITOR").ok()
        .unwrap_or_else(|| "nano".to_string());

    // Split editor string into command and args (e.g., "code --wait")
    let mut parts = editor_string.split_whitespace();
    let editor_bin = parts.next().expect("Empty editor command");
    let editor_args: Vec<String> = parts.map(|s| s.to_string()).collect();

    // Full command to run
    let mut cmd = Command::new(editor_bin);
    for arg in &editor_args {
        cmd.arg(arg);
    }
    cmd.arg(path);

    let full_command = format!(
        "{} {} {}",
        editor_bin,
        editor_args.join(" "),
        path.to_string_lossy()
    );

    // Try to run it
    match cmd.status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("Editor exited with status: {}", status);
            }
        }
        Err(err) => {
            eprintln!("Failed to run editor command: {}\nError: {}", full_command, err);
            std::process::exit(1);
        }
    }
}

fn make_executable(path: &PathBuf) {
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

fn create_script() {
    print!("Enter script name: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    let script_path = get_scripts_dir().join(name);

    // If file doesn't exist yet, write default shebang and description placeholder
    if !script_path.exists() {
        fs::write(&script_path, "#!/bin/bash\n# description: (no description)\n").expect("Failed to write initial script");
    }

    open_in_editor(&script_path);
    make_executable(&script_path);
    println!("Script '{}' created at {:?}", name, script_path);
}

fn list_scripts() {
    let dir = get_scripts_dir();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                let description = extract_description(&path);
                println!("{:<20} {}", name.to_string_lossy(), description);
            }
        }
    }
}

fn remove_script(name: &str) {
    let path = get_scripts_dir().join(name);
    if path.exists() {
        fs::remove_file(&path).unwrap();
        println!("Removed script '{}'", name);
    } else {
        eprintln!("Script '{}' not found", name);
    }
}

fn edit_script(name: &str) {
    let path = get_scripts_dir().join(name);
    if path.exists() {
        open_in_editor(&path);
    } else {
        eprintln!("Script '{}' not found", name);
    }
}

fn run_script(name: &str, args: &[String]) {
    let path = get_scripts_dir().join(name);
    if !path.exists() {
        eprintln!("Script '{}' not found", name);
        exit(1);
    }

    let result = Command::new(&path)
        .args(args)
        .status();

    match result {
        Ok(status) => {
            if !status.success() {
                eprintln!("Script exited with non-zero status: {}", status);
                exit(status.code().unwrap_or(1));
            }
        }
        Err(err) => {
            if let Some(8) = err.raw_os_error() {
                eprintln!(
                    "Failed to execute '{}': Exec format error.\n\
                     Hint: Make sure the script starts with a valid shebang line (e.g., #!/bin/bash)",
                    name
                );
            } else {
                eprintln!("Failed to run script '{}': {}", name, err);
            }
            exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
    } else {
        match args[0].as_str() {
            "ls" => list_scripts(),
            "create" => create_script(),
            "edit" => {
                if args.len() < 2 {
                    eprintln!("Usage: fastbash edit <script>");
                    exit(1);
                }
                edit_script(&args[1]);
            }
            "rm" => {
                if args.len() < 2 {
                    eprintln!("Usage: fastbash rm <script>");
                    exit(1);
                }
                remove_script(&args[1]);
            }
            "help" | "--help" | "-h" => print_help(),
            script_name => run_script(script_name, &args[1..]),
        }
    }
}

