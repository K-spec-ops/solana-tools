mod importscript;
use importscript::*;

fn allowed_decimals(bound: f64)-> impl Fn(&str) -> Result<f64, String>+ Clone {
    move |s: &str| {
        let num: f64= s.parse::<f64>().map_err(|_| format!("'{}' isn't a valid float", s))?;
        if 0.0< num && num< bound {
            Ok(num)
        } else {
            Err(format!("{} isn't within the bounds (0, {})", num, bound))
        }
    }
}

fn allowed_files(s: &str)-> Result<PathBuf, String> {
    let path: PathBuf= s.parse::<PathBuf>().unwrap(); // could use PathBuf::from(s) but I like that this is consistent with above
    if !path.try_exists().unwrap_or(false) {
        return Err("This file does not exist or could not be opened".to_string()) // should use into() IF there is variable assignment
                                                                                  // only use into() if you can tell the compiler the data type, for inference use to_string()
    }
    
    let ext: Option<&OsStr>= path.extension();
    if ext== Some(OsStr::new("lst")) {
        Ok(path)
    } else {
        Err(format!("You attached a .{} file, not a .lst file", ext.map(|x| x.to_string_lossy().to_string())
                                                                .unwrap_or("<no extension>".to_string())))
    }
}

fn allowed_threads(s: &str)-> Result<usize, String> {
    let num: usize= s.parse::<usize>().map_err(|_| format!("{} is not a valid integer", s))?;
    let count: usize= available_parallelism().expect("Could not extract CPU count").get();
    if num<= count {
        Ok(num)
    } else {
        Err(format!("You specified {} workers, but your machine only has {} cores", num, count))
    }
}
   // let count: usize= available_parallelism().map(|x| x.get())
                                                       // .map_err(|_| format!("Could not extract CPU count")).unwrap();

fn trimnw(s: &str)-> &str {
    let s: &str= s.trim_matches(|c: char| c.is_whitespace() || c== '\0');
    s
}

fn get_user_input()-> String {
    let mut userinp: String= String::new();
    stdin().read_line(& mut userinp).expect("Could not read user input");
    trimnw(userinp.as_str()).to_string()
}

fn count_bytes(s: String) -> Result<String, String> {
    let decoded: Vec<u8>= decode(&s).into_vec()
                        .map_err(|_| format!("I couldn't convert the string ({}) into Base58.", s).to_string())?;
    
    if decoded.len()== 32 {
        Ok(s)
    } else {
        Err(format!("I could not recognize the address {}.", s))
    }
}

#[derive(Parser, Debug)]
#[command(version, 
          about= env!("CARGO_PKG_DESCRIPTION"),
          long_about= concat!(env!("CARGO_PKG_DESCRIPTION"), 
                       " You can switch to a strategy based solely on \
                       time duration by omitting the '-s' and '-p' flags. The \
                       '-t' flag will define the amount of time between trades. This script also \
                        has the capability to trade user-chosen tokens, either by inputting a single \
                        token address or compiling token addresses in a .lst file."),
          override_usage= "sniper-trader --stop <SL> --profit <TP> --time <TIME> --file <LST> --workers <NUM> --cloud"
        )]
struct Args {
    // f64::MAX
    #[arg(short= 't', long= "time", default_value_t= 30.0, required= true)] 
    /// Number of seconds after which to sell the token
    time: f64,

    /// Stop-loss in decimal percentage
    #[arg(short= 's', long= "stop", requires= "tp", value_parser= allowed_decimals(1.0))]
    sl: Option<f64>,

    /// Take-profit in decimal percentage
    #[arg(short= 'p', long= "profit", requires= "sl")]
    tp: Option<f64>,

    /// Attach .lst file of token addresses to trade
    #[arg(short= 'f', long= "file", value_parser= allowed_files)]
    lst: Option<PathBuf>,
    
    /// Number of concurrent workers
    #[arg(short= 'w', long= "workers", value_parser= allowed_threads)]
    num: Option<usize>,

    /// Run the script on an EC2 instance
    #[arg(short= 'c', long= "cloud", action= ArgAction::SetTrue)]
    ec: bool

    
}

fn main() {
    // env_logger::Builder::new()
    //        .filter_level(Warn).init();
    let args: Args= Args::parse(); 
    let token_res: Vec<String>= if args.lst.is_none() {
            println!("I see that you haven't attached a .lst file. \
                      Would you like to trade a single address? Type 'Y' for Yes and 'N' for No.");
            loop {
                match get_user_input().to_uppercase().as_str() { // as_str() resolves type mismatch between &str and String
                "Y"=> {
                    println!("Please enter the token address.");
                    break loop {
                        match count_bytes(get_user_input()) {
                            Ok(token)=> break Vec::from([token]),
                            Err(msg)=> {
                                println!("{msg} Solana addresses are fixed to be 32 bytes long. /
                                         Please try again.");
                                continue
                            }
                        }
                    };
                },
                "N"=> break Vec::new(),
                _=> {
                    println!("I can't understand your input. Please try again.");
                    continue
                }};
            }
        } else {
            let mut result: Vec<String>= Vec::new();
            let path: PathBuf= args.lst.unwrap();
            for line in read_to_string(&path).
                        unwrap_or_else(|_| panic!("Unable to read {}", path.to_string_lossy())).lines() {
                                            match count_bytes(trimnw(line).to_string()) {
                                                Ok(line)=> result.push(line),
                                                Err(msg)=> println!("{}: {msg} I'm not going to add it.", "WARNING".yellow())
                                            }
                                        }
            result          
        };
    

    println!("This is token: {:?}", token_res);
}
    


