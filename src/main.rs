mod importscript;

use importscript::*;

fn allowed_decimals(bound: f64) -> impl Fn(&str) -> Result<f64, String>+ Clone {
    move |s: &str| {
        let num: f64= s.parse::<f64>().map_err(|_| format!("'{}' isn't a valid float", s))?;
        if 0.0< num && num< bound {
            Ok(num)
        } else {
            Err(format!("{} isn't within the bounds (0, {})", num, bound))
        }
    }
}

fn allowed_files(s: &str) -> Result<PathBuf, String> {
    let path: PathBuf= s.parse::<PathBuf>().unwrap(); // could use PathBuf::from(s) but I like that this is consistent with above
    if !path.try_exists().unwrap_or(false) {
        return Err("This file does not exist or could not be opened".to_string()) // should use into() IF there is variable assignment
    }
    
    let ext: Option<&OsStr>= path.extension();
    if ext== Some(OsStr::new("lst")) {
        Ok(path)
    } else {
        Err(format!("You attached a .{} file, not a .lst file", ext.map(|x| x.to_string_lossy().to_string())
                                                                .unwrap_or("<no extension>".to_string())))
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
          override_usage= "sniper-trader --stop <SL> --profit <TP> --time <TIME> --file <LST> --cloud"
        )]
struct Args {
    // value_parser= allowed_decimals(f64::MAX)
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
    lst: PathBuf,
    
    /// Run the script on an EC2 instance
    #[arg(short= 'c', long= "cloud", action= ArgAction::SetTrue)]
    ec: bool
}

fn main() {
    let _args: Args= Args::parse(); 
    //if !args.
}

