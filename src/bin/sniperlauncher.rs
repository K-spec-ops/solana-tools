use solana_tools::*;
use solana_tools::utils::*;

// cargo install --path /home/kabir/Dropbox/sniper-trader, to get 'sniper-trader' to run anywhere by just doing 'sniper-trader'
// cargo install --path /home/kabir/Dropbox/sniper-trader --force, IF you need to reinstall (new code, etc.)

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
    let path: PathBuf= PathBuf::from(s); // could also use the parse() method above but that is unwieldy
                                         // ::from() is reserved for lossless infallible conversions!
    if !path.try_exists().unwrap_or_else(|_| false) {
        return Err("This file does not exist or could not be opened".to_string()) // should use into() IF there is variable assignment
                                                                                  // only use into() if you can tell the compiler the data type, for inference use to_string()
    }
    
    let ext: Option<&OsStr>= path.extension();
    if ext== Some(OsStr::new("lst")) {
        Ok(path)
    } else {
        Err(format!("You attached a .{} file, not a .lst file", ext.map(|x| x.to_string_lossy().to_string())
                                                                .unwrap_or_else(|| "<no extension>".into())))
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

fn allowed_times(s: &str)-> Result<String, String> {
    s[0..s.len()-1].parse::<f64>().map_err(|_| format!("'{}' isn't a valid float", s))?;
    if !(s.ends_with('m') || s.ends_with('h')) {
        return Err(format!("'{}' is an invalid suffix, only 'm' or 'h' are permitted", &s[s.len()-1..]))
    };

    Ok(s.into()) 
}

fn get_user_input()-> String {
    let mut userinp: String= String::new();
    stdin().read_line(& mut userinp).expect("Could not read user input");
    trimnw(userinp.as_str()).to_string()
}

fn count_bytes(s: String)-> Result<String, String> {
    let decoded: Vec<u8>= decode(&s).into_vec()
                        .map_err(|_| format!("I couldn't convert the string ({}) into Base58.", s).to_string())?;
    
    if decoded.len()== 32 {
        Ok(s)
    } else {
        Err(format!("I could not recognize the address {}.", s))
    }
}

#[tokio::main]
async fn create_instance(imageid: &str, instancetype: InstanceType, 
                            a: &Args)-> Result<(), Box<dyn Error>> { // Use Box<...> when the function can return mutliple error types
    let placement: Placement= Placement::builder().availability_zone("us-east-1a").build();
    let config: SdkConfig= load_defaults(BehaviorVersion::latest()).await;

    match sts_Client::new(&config).get_caller_identity().send().await {
        Ok(x)=> println!("AWS credentials successfully validated!\n   User ID: {}\n   Account ID: {}\n   ARN: {}", 
                                    x.user_id.unwrap(), x.account.unwrap(), x.arn.unwrap()),
        Err(_)=> panic!("Could not validate AWS credentials. Please check that they are in your environment")
    }
    let client: Client= Client::new(&config);
    let mut argstr: String= format!("sniper-trader -e {} -t {} -w {}", a.tot, a.time, a.num);
    if a.sl.is_some() || a.tp.is_some() {
        argstr.push_str(format!(" -s {} -p {}", a.sl.unwrap(), a.tp.unwrap()).as_str());
    }
    if a.lst.is_some() {argstr.push_str(format!(" -f {}", a.lst.as_ref().unwrap().display()).as_str())};
    if a.lg {argstr.push_str(" -l")}; 
    // add '&' to things that aren't Option<T> since they don't have Copy. If you don't add &, it will compile BUT you won't be able to use it 


    // git clone https://github.com/K-spec-ops/solana-tools.git\n\
    //                               cd solana-tools\n\
    //                               cd solana-tools\n\
    let userdata: String= format!("#!/bin/bash\n\
                                   export HOME=/root\n\
                                   sudo dnf install git gcc -y\n\
                                   curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y\n\
                                   . \"$HOME/.cargo/env\"\n\
                                   git clone https://github.com/K-spec-ops/solana-tools.git\n\
                                   cd solana-tools\n\
                                   {}", argstr); // shutdown now
    let info: RunInstancesOutput= client.run_instances().image_id(imageid)
                          .max_count(1)
                          .min_count(1)
                          .placement(placement)
                          .instance_initiated_shutdown_behavior(ShutdownBehavior::Terminate)
                          .instance_type(instancetype)
                          .user_data(STANDARD.encode(userdata)).send().await?; // '?' here since the default aws errors are prob better than my own

    let instance: &Instance= info.instances.as_ref().expect("Could not extract instance details").first().expect("No instance returned");                      
    let platform: DescribeImagesOutput= client.describe_images().image_ids(imageid).send().await?;
    println!("AWS instance partitioned.\n   Reservation ID: {}\n   Architecture: {}\n   State: {}\n   Platform: {}\n   Instance Type: {}\n   Availability Zone: {} 
                ", info.reservation_id.as_deref().unwrap_or_else(|| "None"), 
                   &instance.architecture.as_ref().map(|x| x.as_str()).unwrap_or_else(|| "None"),
                   match instance.state.as_ref().and_then(|x| x.code) {
                                                Some(0)=> "pending",
                                                Some(16)=> "running",
                                                Some(32)=> "shutting-down",
                                                Some(48)=> "terminated",
                                                _=> "None"},
                   platform.images.as_deref().unwrap_or_default().first()
                                    .map_or("None", |x| x.platform_details.as_deref().unwrap_or("None")),
                   instance.instance_type.as_ref().map(|x| x.as_str()).unwrap_or_else(|| "None"),
                   instance.placement.as_ref().and_then(|x| x.availability_zone.as_deref()).unwrap_or_else(|| "None"));
    
    Ok(())

}

fn launcher(a: &Args, token_vec: &Vec<String>, ttime: f64)-> () { // consider using enum here...
    if a.ec {
        let ami: &str= "ami-0bd3fbcdc633a1b1a"; // Amazon Linux 2023 kernel-6.18 AMIA, until June 2029
        let instance: InstanceType= InstanceType::T2Small;
        create_instance(ami, instance, a).unwrap()
    } else {
        let log_path: PathBuf= if a.num>1 || !a.lg {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        } else {
            PathBuf::new()};
        let trade_path: PathBuf= env::current_dir().expect("Couldn't extract current dir path")
                                                  .parent().unwrap().join("/bin/executetrades.rs");
        println!("{:?}", trade_path);
        // let trade_path: PathBuf= env::current_exe()
        //                 .expect("Couldn't extract current .exe path")
        //                 .parent().expect("Couldn't get parent dir from .exe path")
        //                 .join("execute-trades");
        let sl: String= a.sl.as_ref().unwrap().to_string();
        let tp: String= a.tp.as_ref().unwrap().to_string();
        let child= Command::new(trade_path.as_path()) // '&' is for borrowing
                                    .args([&a.time.to_string(), &sl, &tp])
                                    .arg(log_path)
                                    .args(token_vec)
                                    .arg(ttime.to_string())
                                    .stdin(Stdio::piped())
                                    .stdout(Stdio::piped())
                                    .spawn()
                                    .expect("Failed to execute child");
        child.wait_with_output().expect("Failed to wait on child");
    }
}

#[derive(Parser, Debug)]
#[command(version, 
          about= "A Rust script to snipe recently graduated pump.fun coins \
                using an RPC and Solana on-chain data.",
          long_about= format!("A Rust script to snipe recently graduated pump.fun coins \
                        using an RPC and Solana on-chain data. You can switch to a strategy based solely on \
                        time duration by omitting the '-s' and '-p' flags. The \
                       '-t' flag will define the amount of time between trades. This script also \
                        has the capability to trade user-chosen tokens, either by inputting a single \
                        token address or compiling token addresses in a .lst file.\n\n\
                        To launch an EC2 instance, you need to add the 'AWS_ACCESS_KEY_ID' \
                        and 'AWS_SECRET_ACCESS_KEY' environment variables to your .bashrc file. Ctrl + Click \
                        {} for more details. You also need to attach the 'AmazonEC2FullAccess' IAM policy.", 
                        "here".hyperlink("https://docs.aws.amazon.com/sdkref/latest/guide/environment-variables.html")),
          override_usage= "sniper-trader --stop <SL> --profit <TP> --time <TIME> --total <TOT> --file <LST> --workers <NUM> --log --cloud"
        )]
struct Args {
    // f64::MAX
    #[arg(short= 'e', long= "total", default_value= "30m", value_parser= allowed_times)] 
    /// Number of minutes (if you append an "m") or hours (if you use "h") to run the bot (e.g. 15m or 24h)
    tot: String,

    #[arg(short= 't', long= "time", default_value_t= 30.0)] 
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
    #[arg(short= 'w', long= "workers", default_value_t= 1, value_parser= allowed_threads)]
    num: usize,

    /// Log messages to a file. Extraneous for '-w' > 1, since each worker will log its messages by default
    #[arg(short= 'l', long= "log", action= ArgAction::SetTrue)]
    lg: bool,

    /// Run the script on an EC2 instance
    #[arg(short= 'c', long= "cloud", action= ArgAction::SetTrue)]
    ec: bool

    
}

fn main() {
    // env_logger::Builder::new()
    //        .filter_level(Warn).init();
    let args: Arc<Args>= Arc::new(Args::parse()); 
    let tot_num: f64= args.tot[0..args.tot.len()-1].parse::<f64>().unwrap();
    let tot_time: f64= if args.tot.ends_with('m') {tot_num* 60.0} else {tot_num* 3600.0};
    let token_res: Arc<Vec<String>>= if args.lst.is_none() { // 'Arc' this
            println!("I see that you haven't attached a .lst file. \
                      Would you like to trade a single address? Type 'Y' for Yes and 'N' for No.");
            loop {
                match get_user_input().to_uppercase().as_str() { // as_str() resolves type mismatch between &str and String
                "Y"=> {
                    println!("Please enter the token address.");
                    break loop {
                        match count_bytes(get_user_input()) {
                            Ok(token)=> break Arc::new(Vec::from([token])),
                            Err(msg)=> {
                                println!("{msg} Solana addresses are fixed to be 32 bytes long. /
                                         Please try again.");
                                continue
                            }
                        }
                    };
                },
                "N"=> break Arc::new(vec!["None".into()]),
                _=> {
                    println!("I can't understand your input. Please try again.");
                    continue
                }};
            }
        } else {
            let mut result: Vec<String>= Vec::new();
            let path: &PathBuf= args.lst.as_ref().unwrap(); // .as_ref() to reference content of args w/o moving it, which causes a problem in the thread
            for line in read_to_string(&path).
                        unwrap_or_else(|_| panic!("Unable to read {}", path.to_string_lossy())).lines() {
                                            match count_bytes(trimnw(line).to_string()) {
                                                Ok(line)=> result.push(line),
                                                Err(msg)=> println!("{}: {msg} I'm not going to add it.", "WARNING".yellow())
                                            }
                                        }
            Arc::new(result)          
        };

    // println!("This is token: {:?}", token_res);
    let handles: Vec<_>= (1..=args.num)
                .map(|i: usize| {let args: Arc<Args>= Arc::clone(&args);
                                 let token_res_clone: Arc<Vec<String>>= Arc::clone(&token_res);
                                 spawn(move || {println!("Starting worker {i}...");
                                               launcher(&args, &token_res_clone, tot_time)})}).collect();
                                               
    for h in handles {
        let output:() = h.join().expect("Worker thread ran into an issue"); // use expect() instead of unwrap_or_else() when you want to panic without needing to format 
                                                             // Nice thing, unwrap_or_else() can use panics AND recoverable errors; expect() only panics
        println!("{:?}", output)
       // let pp: String= String::from_utf8_lossy(&output.stdout).lines().last().unwrap_or("").into();
       // println!("{pp}"); <-- to get last line from child
    };
    hello();
    println!("All threads are finished!!")

}
    


