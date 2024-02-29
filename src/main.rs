use anyhow::{anyhow, Error, Result};
use clap::Parser;
use console::Term;
use std::time::SystemTime;
use std::{
    io::{BufRead, BufReader, Write},
    sync::{Arc, Mutex},
    thread,
};

// The higher the number, the slower the game
const GAME_SPEED_MULTIPLIER: f64 = 1.3;
const DEFAULT_MIPSY_PATH: &str = "/home/cs1521/bin/mipsy";

#[derive(Parser, Debug)]
#[clap(name = "railroad-runners")]
struct Cli {
    /// The file name of the railroad-runners assignment.
    file_name: String,

    /// The path to the mipsy executable.
    #[arg(long, value_name = "mipsy_path")]
    mipsy_path: Option<String>,
}

#[derive(Clone, Debug)]
struct Args {
    file_name: String,
    mipsy_path: String,
}

fn main() -> Result<()> {
    run_game(&parse_args(), get_seed()?)?;
    Ok(())
}

fn get_seed() -> Result<u32> {
    print!("Enter a non-zero seed: ");
    std::io::stdout().flush()?;

    let mut seed = String::new();
    std::io::stdin().read_line(&mut seed)?;
    let seed = seed
        .trim()
        .parse()
        .map_err(|_| anyhow!("Failed to parse seed"))?;

    if seed == 0 {
        Err(anyhow!("Seed can't be zero"))
    } else {
        Ok(seed)
    }
}

fn print_thread(stdout: BufReader<&mut std::process::ChildStdout>) {
    for line in stdout.lines() {
        println!("{}", line.unwrap());
    }
}

fn tick_thread(
    game_thread: Arc<Mutex<thread::ScopedJoinHandle<()>>>,
    stdin: Arc<Mutex<&mut std::process::ChildStdin>>,
    start_time: &SystemTime,
) {
    loop {
        if game_thread.lock().unwrap().is_finished() {
            break;
        }

        stdin.lock().unwrap().write_all(b"\'\n").unwrap();

        let now = SystemTime::now();
        let elapsed = now.duration_since(*start_time).unwrap().as_secs_f64();

        const BASE: f64 = 10.0;
        let num = f64::log(if elapsed < BASE { BASE } else { elapsed }, BASE);
        let time_to_sleep = f64::min(1.0, GAME_SPEED_MULTIPLIER / num);

        println!("num: {}", num);
        println!("tts: {}", time_to_sleep);
        std::thread::sleep(std::time::Duration::from_secs_f64(time_to_sleep));
    }
    println!("Press any key to exit");
}

fn input_thread(
    game_thread: Arc<Mutex<thread::ScopedJoinHandle<()>>>,
    stdin: Arc<Mutex<&mut std::process::ChildStdin>>,
) {
    const VALID_CHARS: &[char] = &['w', 'a', 's', 'd', 'q'];

    loop {
        let term = Term::stdout();
        if game_thread.lock().unwrap().is_finished() {
            break;
        }

        let input = term.read_char().unwrap();
        if VALID_CHARS.contains(&input) {
            if let Err(_) = stdin
                .lock()
                .unwrap()
                .write_all(format!("{}\n", input).as_bytes())
            {
                break;
            }
        }
    }
}

fn run_game(args: &Args, seed: u32) -> Result<()> {
    println!("Starting Railroad Runners...");

    let mut child = std::process::Command::new(&args.mipsy_path)
        .arg(&args.file_name)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().ok_or(Error::msg("No stdin"))?;
    let stdout = BufReader::new(child.stdout.as_mut().ok_or(Error::msg("No stdout"))?);

    stdin.write_all(format!("{}\n", seed).as_bytes())?;

    let start_time = SystemTime::now();

    thread::scope(|scope| {
        // Grabs output from the game
        let handle = Arc::new(Mutex::new(scope.spawn(|| print_thread(stdout))));

        let stdin = Arc::new(Mutex::new(stdin));
        let stdin_mutex = stdin.clone();
        let game_thread = handle.clone();
        // Tick thread, advances the game state every so often
        scope.spawn(|| tick_thread(game_thread, stdin_mutex, &start_time));

        let game_thread = handle.clone();
        let stdin_mutex = stdin.clone();
        // Input thread, listens for user input
        scope.spawn(|| input_thread(game_thread, stdin_mutex));
    });

    child.wait()?;
    Ok(())
}

fn parse_args() -> Args {
    let args = Cli::parse();
    Args {
        file_name: args.file_name,
        mipsy_path: args.mipsy_path.unwrap_or(DEFAULT_MIPSY_PATH.to_string()),
    }
}
