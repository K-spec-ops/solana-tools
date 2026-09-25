use solana_tools::*;
use solana_tools::utils::*;

// cargo install --path /home/kabir/Dropbox/sniper-trader, to get 'sniper-trader' to run anywhere by just doing 'sniper-trader'
// cargo install --path /home/kabir/Dropbox/sniper-trader --force, IF you need to reinstall (new code, etc.)
// Look at building with Github actions

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
    let n: usize= stdin().read_line(&mut userinp).expect("Could not read user input");
    if n== 0 {
        return "N".to_string(); // no terminal (e.g. EC2 cloud-init) so skip the prompt
    }
    trimnw(&userinp).to_string()
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

async fn add_policies(config: &SdkConfig, uuid: String)-> Result<String, Box<dyn Error>> {
    let policy: &str= "{
                \"Version\": \"2012-10-17\",
                \"Statement\": [{
                    \"Effect\": \"Allow\",
                    \"Action\": \"{}:*\",
                    \"Resource\": \"*\"}]}"; // not secure, but it this should only be ran on a personal account

    let client: iam_Client= iam_Client::new(config);
    let user: String= match client.get_user().send().await {
        Ok(x)=> x.user.expect("Could not extract user info").user_name,
        Err(_)=> panic!("IAM user object did not initialize (Are you using root credentials? / 
                                                        Don't do that. Please create an IAM user for this script)")
    };
    println!("Your IAM user name is {}", user);

    let ssm_pol_obj: CreatePolicyOutput= client.create_policy()
                          .policy_name(format!("{}{}", "ssm_policy_", uuid))
                          .policy_document(policy.replace("{}", "ssm")).send().await?;
    println!("Created an SSM policy with the name {}", ssm_pol_obj.policy.expect("Could not extract SSM policy info")
                                                                    .policy_name.unwrap_or_else(|| "<unavailable>".to_string()));                                                                                                          

    let s3_pol_obj: CreatePolicyOutput= client.create_policy()
                          .policy_name(format!("{}{}", "s3_policy_", uuid))
                          .policy_document(policy.replace("{}", "s3")).send().await?;
    println!("Created an S3 policy with the name {}", s3_pol_obj.policy.expect("Could not extract S3 policy info")
                                                                    .policy_name.unwrap_or_else(|| "<unavailable>".to_string()));

    let ec2_pol_obj: CreatePolicyOutput= client.create_policy()
                          .policy_name(format!("{}{}", "ec2_policy_", uuid))
                          .policy_document(policy.replace("{}", "ec2")).send().await?;
    println!("Created an EC2 policy with the name {}", ec2_pol_obj.policy.expect("Could not extract EC2 policy info")
                                                                    .policy_name.unwrap_or_else(|| "<unavailable>".to_string()));

    let iam_pol_obj: CreatePolicyOutput= client.create_policy()
                          .policy_name(format!("{}{}", "iam_policy_", uuid))
                          .policy_document(policy.replace("{}", "iam")).send().await?;
    println!("Created an IAM policy with the name {}", iam_pol_obj.policy.expect("Could not extract IAM policy info")
                                                                    .policy_name.unwrap_or_else(|| "<unavailable>".to_string()));
    
    Ok(user)
    
}

#[tokio::main]
async fn create_instance(imageid: &str, instancetype: InstanceType, 
                            a: &Args, token_vec: &Vec<String>)-> Result<(), Box<dyn Error>> { // Use Box<...> when the function can return mutliple error types
    let placement: Placement= Placement::builder().availability_zone("us-east-1a").build();
    let config: SdkConfig= load_defaults(BehaviorVersion::latest()).await;

    match sts_Client::new(&config).get_caller_identity().send().await {
        Ok(x)=> {println!("AWS credentials successfully validated!\n   User ID: {}\n   Account ID: {}\n   ARN: {}", 
                                    x.user_id.as_ref().unwrap(), x.account.unwrap(), x.arn.as_ref().unwrap());
                                    x.arn.unwrap()},
        Err(_)=> panic!("Could not validate AWS credentials. Please check that they are in your environment")
    }; // println! consumes owned strings

    add_policies(&config, Uuid::new_v4().to_string()).await?;

    let client: Client= Client::new(&config);
    let mut argstr: String= format!("sniper-trader -e {} -t {} -w {}", a.tot, a.time, a.num);
    if a.sl.is_some() || a.tp.is_some() {
        let _= write!(&mut argstr, " -s {} -p {}", a.sl.unwrap(), a.tp.unwrap());
    }
    if a.lg {argstr.push_str(" -l")}; 
    
    let mut s3_line_1= String::new();
    let mut s3_line_2: String= String::new();
    if !token_vec.is_empty() && token_vec.first().unwrap()!= "None" {
        let s3: s3_Client= s3_Client::new(&config);
        let pathstr: &str= "tokens.lst";
        let bucket: String= format!("snipertrader-tokens-{}", Uuid::new_v4().to_string());
        s3.create_bucket().bucket(&bucket).send().await?;
        let putobj: PutObjectOutput= s3.put_object()
                                       .bucket(&bucket)
                                       .key(pathstr)
                                       .body(ByteStream::from((token_vec.join("\n")+ "\n").into_bytes()))
                                       .send().await?;
        let expiry: String= putobj.expiration.unwrap_or_else(|| "None".into());
        println!("Adding '{}' to S3...\n   Expiration date: {}", pathstr, expiry);
        argstr.push_str(" -f /root/copy.lst");
        write!(&mut s3_line_1, "aws s3 cp s3://{}/{} /root/copy.lst\n", bucket, pathstr)?; 
        write!(&mut s3_line_2, "aws s3 rb s3://{} --force\n", bucket)?;
    } // use write! to change EXISTING mut strings that need formatting/aren't static, otherwise for EXISTING strings use push_str for performance
    // add '&' to things that aren't Option<T> since they don't have Copy. If you don't add &, it will compile BUT you won't be able to use it 

    let userdata: String= format!("#!/bin/bash\n\
                                   export HOME=/root\n\
                                   sudo dnf install git gcc -y\n\
                                   {}\
                                   curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y\n\
                                   . \"$HOME/.cargo/env\"\n\
                                   git clone https://github.com/K-spec-ops/solana-tools.git\n\
                                   cd solana-tools\n\
                                   cargo install --path .\n\
                                   {}\n\
                                   {}", s3_line_1, argstr, s3_line_2); // shutdown now
    let info: RunInstancesOutput= client.run_instances().image_id(imageid)
                          .max_count(1)
                          .min_count(1)
                          .placement(placement)
                          .instance_initiated_shutdown_behavior(ShutdownBehavior::Terminate)
                          .instance_type(instancetype)
                          .user_data(STANDARD.encode(userdata)).send().await?; // '?' here since the default aws errors are prob better than my own

    let instance: &Instance= info.instances.as_ref().expect("Could not extract instance details").first().expect("No instance returned");                      
    let instanceid: &String= instance.instance_id.as_ref().unwrap_or_else(|| panic!("Could not extract instance id"));
    let platform: DescribeImagesOutput= client.describe_images().image_ids(imageid).send().await?;
    client.wait_until_instance_running().instance_ids(instanceid)
                                        .wait(Duration::from_mins(3)).await?; // remember, '?' automates error handling here
    let current: DescribeInstancesOutput= client.describe_instances().instance_ids(instanceid).send().await?; 
    println!("AWS instance partitioned.\n   Reservation ID: {}\n   Architecture: {}\n   State: {}\n   Platform: {}\n   Instance Type: {}\n   Availability Zone: {} 
                ", info.reservation_id.as_deref().unwrap_or_else(|| "None"), 
                   &instance.architecture.as_ref().map(|x| x.as_str()).unwrap_or_else(|| "None"),
                   match current.reservations.as_deref().unwrap_or_default().iter()
                                             .flat_map(|x| x.instances.as_deref().unwrap_or_default())
                                             .next().and_then(|x| x.state.as_ref()).and_then(|x| x.code) {
                                                Some(0)=> "pending",
                                                Some(16)=> "running",
                                                Some(32)=> "shutting down",
                                                Some(48)=> "terminated",
                                                _=> "None"},
                   platform.images.as_deref().unwrap_or_default().first()
                                    .map_or("None", |x| x.platform_details.as_deref().unwrap_or("None")),
                   instance.instance_type.as_ref().map(|x| x.as_str()).unwrap_or_else(|| "None"),
                   instance.placement.as_ref().and_then(|x| x.availability_zone.as_deref()).unwrap_or_else(|| "None"));
    
    Ok(())

}

fn launcher(index: usize, a: &Args, token_vec: &Vec<String>, ttime: f64)-> () { // consider using enum here...
    if a.ec {
        let ami: &str= "ami-0eb45f74aa8a20238"; // Amazon Linux 2023 kernel-6.18 AMIA, until June 2029 (Arm64)
        let instance: InstanceType= InstanceType::R6gLarge; // Use 'X2gdLarge' once you get vCPU access
        create_instance(ami, instance, a, token_vec).unwrap()
    } else {
        let log_path: PathBuf= if a.num>1 || a.lg {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        } else {
            PathBuf::new()};
        let trade_path: PathBuf= env::current_exe().expect("Couldn't extract current dir path")
                                                  .parent().unwrap().join("executetrades");
        println!("{:?}", trade_path);
        // let trade_path: PathBuf= env::current_exe()
        //                 .expect("Couldn't extract current .exe path")
        //                 .parent().expect("Couldn't get parent dir from .exe path")
        //                 .join("execute-trades");
        let sl: String= a.sl.as_ref().unwrap().to_string();
        let tp: String= a.tp.as_ref().unwrap().to_string();
        let child= Command::new(trade_path.as_path()) // '&' is for borrowing
                                    .args([&a.time.to_string(), &sl, &tp])
                                    .arg(index.to_string())
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
                        {} for more details.\n\nNOTE: The AWS SDKs are very large crates, and compilation can be killed by the OS \
                        if there isn't enough memory (> than 8 GiB). If you have an older desktop or a laptop, it is recommended to \
                        either compile on another machine, add more swap memory, or build the program \
                        using an EC2 instance, create a \"target/release/\" directory in your cloned repo, then copy the binaries over to said directory.", 
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
                "N"=> break Arc::new(Vec::new()),
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
    let handles: Vec<_>= (1..=args.num).map(|i: usize| {let args: Arc<Args>= Arc::clone(&args);
    let token_res_clone: Arc<Vec<String>>= Arc::clone(&token_res);
    spawn(move || {println!("Starting worker {i}...");
                   launcher(i, &args, &token_res_clone, tot_time)})}).collect();
                                         
    for h in handles {
        h.join().expect("Worker thread ran into an issue"); // use expect() instead of unwrap_or_else() when you want to panic without needing to format 
                                                             // Nice thing, unwrap_or_else() can use panics AND recoverable errors; expect() only panics
       // let pp: String= String::from_utf8_lossy(&output.stdout).lines().last().unwrap_or("").into();
       // println!("{pp}"); <-- to get last line from child
    };

    println!("All threads are finished!")

}
    


