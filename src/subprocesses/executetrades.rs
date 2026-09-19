use crate::*;
use crate::utils::*;

fn main() {
    let args: Vec<String>= env::args().collect();

    let time: &f64= &args[1].parse::<f64>().unwrap();
    let sl: &f64= &args[2].parse::<f64>().unwrap();
    let tp: &f64= &args[3].parse::<f64>().unwrap();
    let ttime: &f64= &args[args.len()- 1].parse::<f64>().unwrap();
    let log_path: &Path= &Path::new(&args[4]);
    let token_vec: &Vec<String>= &args[5..args.len()- 1].to_vec();


    println!("Here is the time: {}\n\
              Here is the stop loss: {}\n\
              Here is the take profit: {}\n\
              Here is the log path: {:?}\n\
              Here is the token vector: {:?}\n\
              Here is the total time: {}", time, sl, tp, log_path, token_vec, ttime);

}
