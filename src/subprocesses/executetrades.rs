use solana_tools::*;
use solana_tools::utils::*;

#[derive(Debug, PartialEq)]
enum InstanceStats {
    X2gdLarge,
    T2Medium
}

impl InstanceStats {
    fn cost(&self, seconds: f64)-> String { // borrowed since 'cost' will consume the value
        match self {
            InstanceStats::X2gdLarge=> format!("{:.2}", (seconds/ 3600.0)* 0.0167),
            InstanceStats::T2Medium=> format!("{:.2}", (seconds/ 3600.0)* 0.0464)
        }
    }
}

fn main() {
    let now: Instant= Instant::now();

    let args: Vec<String>= env::args().collect();

    let time: &f64= &args[1].parse::<f64>().unwrap(); // the 0 index is usually just the child processes name; discard
    let sl: &f64= &args[2].parse::<f64>().unwrap();
    let tp: &f64= &args[3].parse::<f64>().unwrap();
    let id: &usize= &args[4].parse::<usize>().unwrap();
    let ttime: &f64= &args[args.len()- 1].parse::<f64>().unwrap();
    let log_path: &Path= &Path::new(&args[5]);
    let token_vec: &Vec<String>= &args[6..args.len()- 1].to_vec();

    let config: Config= ConfigBuilder::new().set_level_color(Level::Error, Some(Color::Red))
                                                        .set_level_color(Level::Info, Some(Color::White))
                                                        .set_level_color(Level::Warn, Some(Color::Yellow))
                                                        .set_level_color(Level::Debug, Some(Color::Cyan))
                                                        .set_max_level(LevelFilter::Info).build();

    if !log_path.is_empty() {
       // let dtime: Vec<&str>= Zoned::now().strftime("%Y:%m:%d").to_string().split(':').collect();
        let dtime: Zoned= Zoned::now();
        WriteLogger::init(LevelFilter::Info, config, 
                          File::create(format!("trader-w{}-{}-{}-{}.log", 
                                                      id, dtime.month(), dtime.day(), dtime.year())).unwrap()).unwrap();
    } else {
        TermLogger::init(LevelFilter::Info, config, TerminalMode::Mixed, ColorChoice::Always).unwrap()
    }
    
    println!("Here is the time: {}\n\
              Here is the stop loss: {}\n\
              Here is the take profit: {}\n\
              Here is the log path: {:?}\n\
              Here is the token vector: {:?}\n\
              Here is the total time: {}", time, sl, tp, log_path, token_vec, ttime);
    
    let elapsed: f64= now.elapsed().as_secs_f64();
    println!("This script executed in {} seconds", elapsed);
    println!("This AWS instance cost ${}", InstanceStats::X2gdLarge.cost(elapsed))

}
