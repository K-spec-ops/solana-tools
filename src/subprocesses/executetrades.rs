use solana_tools::*;
use solana_tools::utils::*;
use tracing::Level;

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


fn main() {
    let now: Instant= Instant::now();

    let args: Vec<String>= env::args().collect();

    let time: &f64= &args[1].parse::<f64>().unwrap(); // the 0 index is usually just the child processes name; discard
    let sl: &f64= &args[2].parse::<f64>().unwrap();
    let tp: &f64= &args[3].parse::<f64>().unwrap();
    let cloud: &String= &args[4];
    let id: &usize= &args[5].parse::<usize>().unwrap();
    let ttime: &f64= &args[args.len()- 1].parse::<f64>().unwrap();
    let log_path: &Path= &Path::new(&args[6]);
    let token_vec: &Vec<String>= &args[7..args.len()- 1].to_vec();
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

    set_global_default(Registry::default().with(fmt_layer).with(LevelFilter::DEBUG)).unwrap();
    
    debug!("Here is the time: {}", time);
    debug!("Here is the stop loss: {}", sl);
    debug!("Here is the take profit: {}", tp);
    debug!("Here is the log path: {:?}", log_path);
    debug!("Here is the token vector: {:?}", token_vec);
    debug!("Here is the total time: {}", ttime);
    warn!("This is a warning message!");
    
    let elapsed: f64= now.elapsed().as_secs_f64();
    info!("This script executed in {} seconds", elapsed);
    if cloud== "true" {
        info!("This AWS instance cost ${}", InstanceStats::R6gLarge.cost(elapsed))
    }

}
