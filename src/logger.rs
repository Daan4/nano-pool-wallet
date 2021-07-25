use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
use time;

static LOGGER: Logger = Logger;

const LOGFILE_COUNT: usize = 5;
const LOGFILES: [LogFile; LOGFILE_COUNT] = [
    LogFile {
        path: "log/rpc.log",
        level: Level::Info,
        targets: ["nano_pool::rpc", "", ""],
    },
    LogFile {
        path: "log/ws.log",
        level: Level::Info,
        targets: ["nano_pool::ws", "", ""],
    },
    LogFile {
        path: "log/wallet.log",
        level: Level::Info,
        targets: ["nano_pool::wallet", "nano_pool::pool", "nano_pool::account"],
    },
    LogFile {
        path: "log/info.log",
        level: Level::Info,
        targets: ["", "", ""],
    },
    LogFile {
        path: "log/error.log",
        level: Level::Error,
        targets: ["", "", ""],
    },
];

pub fn init() -> Result<(), SetLoggerError> {
    // Create and empty log files for session
    for log in LOGFILES {
        log.init();
    }

    // Set up logger
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

struct LogFile {
    path: &'static str,
    level: Level,
    targets: [&'static str; 3],
}

impl LogFile {
    fn init(&self) {
        let filepath = Path::new(self.path);
        // Create log dir
        let folderpath = filepath.parent();
        match folderpath {
            Some(path) if !path.exists() => {
                fs::create_dir(path).unwrap();
            }
            _ => {}
        }

        // Create or empty log file
        if filepath.exists() {
            fs::remove_file(filepath).unwrap();
        }
        File::create(filepath).unwrap();
    }

    fn write(&self, log: &String) {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(self.path)
            .unwrap();
        write!(file, "{}", log).unwrap();
    }
}

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Always print to stdout, and log to the following files:
    /// rpc
    /// ws
    /// wallet
    /// TRACE
    /// DEBUG
    /// INFO
    /// WARN
    /// ERROR
    fn log(&self, record: &Record) {
        let message = format!(
            "{} {} {}\n",
            time::strftime("%Y/%m/%d %H:%M:%S.%f", &time::now()).unwrap(),
            record.level(),
            record.args()
        );

        // Log to files
        let level = record.level();
        let target = record.target();
        for log in LOGFILES {
            if log.level >= level && (log.targets.contains(&target) || log.targets[0] == "") {
                log.write(&message);
            }
        }

        // Log to stdout
        println!("{}", message);
    }

    fn flush(&self) {}
}
