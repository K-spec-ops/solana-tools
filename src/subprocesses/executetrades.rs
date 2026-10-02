use solana_tools::*;
use solana_tools::utils::*;

// CREDIT CONSUMPTION:
// get account balance (getBalance)--> 1 credit

const DEVNET: &str= "https://devnet.helius-rpc.com/?api-key=";
const MAINNET: &str= "https://mainnet.helius-rpc.com/?api-key=";
const LOG_LEVEL: LevelFilter= LevelFilter::INFO;
const SOL_MINT: &str= "So11111111111111111111111111111111111111111";

#[derive(Debug, PartialEq)]
enum InstanceStats {
    X2gdLarge,
    R6gLarge,
}

impl InstanceStats {
    fn cost(&self, seconds: f64)-> String { // borrowed since 'cost' will consume the value
        match self {
            InstanceStats::X2gdLarge=> format!("{:.2}", (seconds/ 3600.0)* 0.0167),
            InstanceStats::R6gLarge=> format!("{:.2}", (seconds/ 3600.0)* 0.1008)
        }
    
    // add more stat operations here 

    }
}

#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>> {
    let api: &str= &var("HELIUS_API_KEY").expect("Couldn't find \"HELIUS_API_KEY\" in your environment");
    let project_id: &str= &var("HELIUS_PROJECT_ID").expect("Couldn't find \"HELIUS_PROJECT_ID\" in your environment");
    let rpc_client: RpcClient= RpcClient::new_with_commitment(
                                            format!("{}{}", DEVNET, api), 
                                            CommitmentConfig::confirmed());
    let mut fee_wrt: Option<Writer<File>>= Some(create_writer("fees.csv")?);
    let mut unit_wrt: Option<Writer<File>>= Some(create_writer("units.csv")?);
   // let _helius: Helius= Helius::new(api, Cluster::MainnetBeta).expect("Couldn't create the Helius client");

    let now: Instant= Instant::now();

    let args: Vec<String>= args().collect();

    let time: &f64= &args[1].parse::<f64>().unwrap(); // the 0 index is usually just the child processes name; discard
    let sl: &f64= &args[2].parse::<f64>().unwrap();
    let tp: &f64= &args[3].parse::<f64>().unwrap();
    let cloud: &String= &args[4];
    let num: &usize= &args[5].parse::<usize>().unwrap();
    let id: &usize= &args[6].parse::<usize>().unwrap();
    let log_path: &Path= &Path::new(&args[7]);
    let secret_key: &String= &args[8];
    let token_vec: &Vec<String>= &args[9..args.len()- 1].to_vec();
    let ttime: &f64= &args[args.len()- 1].parse::<f64>().unwrap();
    
    let dform: &[BorrowedFormatItem<'_>]= format_description!("[hour]:[minute]:[second]");

    let format= format().compact().with_ansi(true).with_level(true)
                                                        .with_timer(LocalTime::new(dform))
                                                        .with_source_location(false)
                                                        .with_line_number(false);
                                                        
    let fmt_layer: Layer<Registry, format::PrettyFields, 
                   format::Format<format::Compact, LocalTime<&[BorrowedFormatItem<'_>]>>, 
                   BoxMakeWriter>= 
                   if !log_path.is_empty() {
                            let dtime: Zoned= Zoned::now();
                            let file: File= File::create(format!("trader-w{}-{}-{}-{}.log", 
                                                id, dtime.month(), dtime.day(), dtime.year()))
                                                       .expect("Could not create log file");
                        Layer::<Registry>::default().event_format(format)
                            .with_writer(BoxMakeWriter::new(file))
                            .fmt_fields(PrettyFields::new())} 
                   else {
                        Layer::<Registry>::default().event_format(format)
                            .with_writer(BoxMakeWriter::new(stdout))
                            .fmt_fields(PrettyFields::new())};

    set_global_default(Registry::default().with(fmt_layer).with(LOG_LEVEL)).unwrap();
    
    let master_keypair: Keypair= Keypair::from_base58_string(secret_key);
    let balance: f64= get_account_balance(master_keypair, rpc_client).await;
    // let keypair: Keypair= if *num> 1 {
    //     let split: f64= (get_account_balance(master_keypair)- 15.0)/ *num as f64; // small integer so it should be fine
    //     token_transfer(master_keypair, Keypair::new(), split)
    // } else {
    //     master_keypair
    // };

    info!("The account balance in sol is: {}", balance);
    
    
    debug!("Here is the time: {}", time);
    debug!("Here is the stop loss: {}", sl);
    debug!("Here is the take profit: {}", tp);
    debug!("Here is the log path: {:?}", log_path);
    debug!("Here is the token vector: {:?}", token_vec);
    debug!("Here is the number of workers: {}", num);
    debug!("Here is the secret key: {}", secret_key);
    debug!("Here is the total time: {}", ttime);
    
    let elapsed: f64= now.elapsed().as_secs_f64();
    info!("This script executed in {} seconds", elapsed);
    if cloud== "true" {
        info!("This AWS instance cost ${}", InstanceStats::R6gLarge.cost(elapsed))
    }

    info!("You have used {} credits so far", query_credits(api, project_id).await);

    Ok(())

}
