use daemonize::Daemonize;
// use env_logger;
use log::{error, info};
// use std::ffi::OsStr;
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

const PID_FILE_PATH: &str = "/var/run/cyberguard.pid";
const WORKING_DIRECTORY: &str = "/";
const WATCH_DIRECTORY: &str = "/private/var/log/";

struct YourConfigType {
    log_file_path: String,
}

struct YourDependenciesType {
    // Add your dependencies here
}

async fn initialize_logger(
    config_path: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file(config_path, Default::default())?;
    info!("Logger initialized");
    Ok(())
}

async fn start_daemon(
    pid_file: &str,
    chown_pid_file: bool,
    working_directory: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    Daemonize::new()
        .pid_file(pid_file)
        .chown_pid_file(chown_pid_file)
        .working_directory(working_directory)
        .start()?;
    info!("Daemonized successfully");
    Ok(())
}

async fn start_application(
    config: &YourConfigType,
    _dependencies: &YourDependenciesType,
) -> std::result::Result<(), io::Error> {
    // Example: Reading from a log file
    let log_content = read_from_file(&config.log_file_path)?;
    info!("Read content from log file: {}", log_content);

    // Add more application logic based on your requirements

    info!("Cyberguard application logic started");
    Ok(())
}

fn read_from_file(file_path: &str) -> std::result::Result<String, io::Error> {
    Ok(format!("Content read from file: {}", file_path))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config_path = "log4rs.yaml";

    initialize_logger(config_path).await?;

    info!("Starting cyberguard ...");

    let pid_file = PID_FILE_PATH;
    let chown_pid_file = true;
    let working_directory = WORKING_DIRECTORY;

    let config = YourConfigType {
        log_file_path: "launchd.log".to_string(),
    };
    let dependencies = YourDependenciesType {
        // Initialize your dependencies here
    };

    start_daemon(pid_file, chown_pid_file, working_directory).await?;
    start_application(&config, &dependencies).await?;

    // Watch directory using fswatch
    watch_directory(WATCH_DIRECTORY).await?;

    Ok(())
}

async fn watch_directory(directory: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new("fswatch");
    command
        .arg("-r")
        .arg(directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let process = command.spawn()?;

    let stdout = process.stdout.expect("Failed to capture fswatch output");
    let stderr = process
        .stderr
        .expect("Failed to capture fswatch error output");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Handle file events in a separate task
    tokio::spawn(async move {
        for line in stdout_reader.lines() {
            if let Ok(file_path) = line {
                info!("File event: {:?}", file_path);
                // Process the file (add your file processing logic here)
                process_file(&file_path).await;
            }
        }
    });

    // Handle errors in a separate task
    tokio::spawn(async move {
        for line in stderr_reader.lines() {
            if let Ok(error_msg) = line {
                error!("fswatch error: {:?}", error_msg);
            }
        }
    });

    // Keep the main task alive
    loop {
        sleep(Duration::from_secs(60)).await; // Sleep for 60 seconds (adjust as needed)
    }
}

async fn process_file(file_path: &str) {
    // Add your specific file processing logic here
    info!("Processing file: {:?}", file_path);
}
