use crate::*;

// use the sdks for everything besides trading logic (unless the it can't be used), where it is cheap to do so...
// TODO: potentially get rid of 'Box' statements to reduce overhead for recoverable errors...let's see if it matters...

const TRIES: usize= 10;
const ADMIN_URL: &str= "https://admin-api.helius.xyz";
const LAMPORTS: f64= 1000000000.0;
const MICROLAMPORTS: f64= 1000000.0;

pub fn trimnw(s: &str)-> &str {
    let s: &str= s.trim_matches(|c: char| c.is_whitespace() || c== '\0');
    s
}

pub fn hello()-> () {
    println!("Hello world! I am utils!")
}

pub fn create_writer(name: &str)-> Result<Writer<File>, Box<dyn Error>> {
    let file: File= OpenOptions::new().write(true).create(true).append(true)
                                                        .open(name)?;
    Ok(csv::Writer::from_writer(file))
    
}

pub async fn req_wrapper(url: String, method: &str, headers: Option<Vec<HashMap<String, String>>>,
                                form: Option<&[(&str, &str)]>, json: Option<HashMap<&str, &str>>)-> req_Response {
    let meth: Method= if method== "GET" {
        Method::GET
    } else if method== "POST" {
        Method::POST
    } else {
        panic!("Invalid request method \"{}\"", method)
    };

    let client: req_Client= req_Client::new();
    let mut request: RequestBuilder= client.request(meth, &url);
    let mut headermap= HeaderMap::new();

    if headers.is_some() {
        let vec_of_maps: Vec<HashMap<String, String>>= headers.unwrap();
        for hash in vec_of_maps {
            for (key, val) in hash {
                if let (Ok(h_name), Ok(h_val)) = (
                    HeaderName::from_bytes(key.to_lowercase().as_bytes()),
                    HeaderValue::from_str(&val.to_lowercase()),
                ) {
                    headermap.insert(h_name, h_val);
                }
            }
        }
        request= request.headers(headermap);

    }

    if json.is_some() { // change these 
        let parsed_json: String= serde_json::to_string(&json).expect("could not serialize hashmap to json");
        request= request.header("Content-Type", "application/json").body(parsed_json);
    }

    if form.is_some() {
        let parsed_array= serde_urlencoded::to_string(&form).expect("could not serialize array to");
        request= request.header("Content-Type", "application/x-www-form-urlencoded").body(parsed_array);
    }

    for i in 1..=TRIES {
        info!("Submitting request to \"{}\", attempt {}", &url, i);
        let attempt: RequestBuilder= request.try_clone().expect("Could not clone request");
        match attempt.send().await {
            Ok(resp)=> match resp.error_for_status() {
                                            Ok(resp)=> return resp,
                                            Err(e)=> {let status: u16= e.status().unwrap().as_u16();
                                                             match status {
                                                                429=> {error!("API calls have been rate-limited (code {}). Retrying...", status);
                                                                        sleep(Duration::from_secs(i.try_into().unwrap())).await;
                                                                        continue},
                                                                400..=499=> {panic!("Unrecoverable client side error (code {}). Examine your params...", status)},
                                                                _=> {error!("Internal server error (code {}). Retrying...", status);
                                                                     continue},
                                                                }
                                                            }
                                                        },                         
            Err(e)=> panic!("Failed to send request: {}", e),
            };
        }
    panic!("Exhausted all {} retry attempts against \"{}\"", TRIES, url);

    }

pub async fn query_credits(api: &str, project: &str)-> f64 {
    let mut headers: HashMap<String, String>= HashMap::new();
    headers.insert("X-Api-Key".to_string(), api.to_string());

    let res: serde_json::Value= req_wrapper(format!("{}/v0/admin/projects/{}/usage", ADMIN_URL, project),
                                                        "GET", Some(vec![headers]), None, None)
                                                        .await.json()
                                                        .await.expect("Could not serialize response object to json");
    
    let credits:f64= res.get("creditsUsed").and_then(|x| x.as_f64()).unwrap_or_else(|| {
                                                                 error!("could not find \"creditsRemaining\" 
                                                                 in the json response. Credit estimate will 
                                                                 be inaccurate"); 0.0});
    
    // info!("You have {} credits...", credits);
    // sleep(Duration::from_secs(5)).await;
    credits
}

pub async fn get_account_balance(account: Keypair, client: RpcClient)-> f64 {
    let balance: u64= match client.get_balance(&account.pubkey()).await {
        Ok(val)=> val,
        Err(e)=> panic!("Could not get account balance: {:?}", e),
    };

    (balance as f64)/ LAMPORTS
}

pub async fn get_priority_fee(client: RpcClient)-> Result<(), Box<dyn Error>> {
    // ceil(compute_unit_price * compute_unit_limit / 1,000,000)
    // simulate the transaction to get "unitsConsumed", log it to csv to determine a CU limit add 10% margin, 
    // do "fee"/"unitsConsumed" to get CU price and log it to determine an appropriate amount, 
    Ok(())
}

pub async fn check_cost(trans: Transaction, method: &str, client: RpcClient, 
                                                writer: &mut Option<Writer<File>>)-> Result<(), Box<dyn Error>> {
    let fee: u64= client.get_fee_for_message(&trans.message).await
                                            .unwrap_or_else(|e| {error!("Could not get the transaction fee, 
                                                                               fee estimate will be inaccurate: {}", e);
                                                                                0})/ LAMPORTS as u64;
    info!("The base fee for the {} method is {} SOL", method, fee);
    if let Some(file_writer)= writer {
        file_writer.write_record(&[method.to_string(), fee.to_string()])?;
        file_writer.flush()?;
    };

    Ok(())
}

pub async fn token_transfer(mover: Keypair, taker: Keypair, mint: Keypair, client: RpcClient, amount: u64)-> 
                                                                Result<(), Box<dyn Error>> {
    // let mover_addy: Option<&Address>= 
        //Some(&get_associated_token_address(&mover.pubkey(), &mint.pubkey()));
    let account: Vec<u8>= client.get_account(&mint.pubkey()).await
                        .unwrap_or_else(|e| panic!("Could not retrieve account details for mint: {}", e)).data;
    let decimals: u8= Mint::unpack(&account)
                    .unwrap_or_else(|e| panic!("Could not retrieve decimals for mint: {}", e)).decimals;

    let instr: message::Instruction= transfer_checked(&token_program_id(), &mover.pubkey(), &mint.pubkey(),
                                &taker.pubkey(), &mover.pubkey(),
                                             &[], amount, decimals)
                        .unwrap_or_else(|e| panic!("Could not parse token transfer instruction: {}", e));
    // "transfer_checked" (or transferChecked) automatically checks the token's mint and decimal count for all transfers

    let blockhash: Hash= client.get_latest_blockhash().await
                                                .unwrap_or_else(|e| panic!("Could not get latest blockhash: {}", e));
    let transaction: Transaction= Transaction::new_signed_with_payer(&[instr], 
                                                                    Some(&mint.pubkey()),
                                                                    &[&mover],
                                                                    blockhash);
    
    client.send_and_confirm_transaction(&transaction).await?;
    Ok(())
}

pub async fn simulate_transaction(client: RpcClient, trans: Transaction, writer: &mut Option<Writer<File>>)-> 
        Result<(u64, f64), Box<dyn Error>> {
    let config: RpcSimulateTransactionConfig= RpcSimulateTransactionConfig {commitment: CommitmentConfig::confirmed().into(),
                                                                        encoding: UiTransactionEncoding::Base64.into(),
                                                                        replace_recent_blockhash: true,
                                                                        sig_verify: true,
                                                                        min_context_slot: None,
                                                                        accounts: None,
                                                                        inner_instructions: false};
    
    let result: Response<RpcSimulateTransactionResult>= client.simulate_transaction_with_config(&trans, config).await?;
    let fee: u64= result.value.fee.unwrap_or_else(|| {error!("Could not get base fee, defaulting to 5000 lamports"); 5000});
    let units: u64= result.value.units_consumed.expect("Could not get compute units consumed");
    let cup: u64= fee/units; 
    let cul: f64= (units as f64)*1.1;
    info!("The compute unit price is {} SOL and the compute unit limit is {} for this transaction", cup/LAMPORTS as u64,  cul);

    if let Some(file_writer)= writer {
        file_writer.write_record(&["cup".to_string(), cup.to_string()])?;
        file_writer.write_record(&["cul".to_string(), cul.to_string()])?;
        file_writer.flush()?;
    };
    
    Ok((cup, cul))
}

pub async fn send_transaction(client: RpcClient)-> Result<(), Box<dyn Error>> {
    let config: RpcSendTransactionConfig= RpcSendTransactionConfig {skip_preflight: true, // "true" if less latency is needed...
                                                                    preflight_commitment: CommitmentConfig::confirmed().commitment.into(),
                                                                    encoding: UiTransactionEncoding::Base64.into(),
                                                                    max_retries: Some(0), // this is recommended?
                                                                    min_context_slot: None
                                                                };
                                                                        
                                                                        
    
    // client.simulate_transaction_with_config()
    client.send_and_confirm_transaction_with_config(transaction, commitment, config)
    Ok(())
}