// code based on https://medium.com/51nodes/exploring-iota-2-0-smart-contracts-in-a-private-network-developing-a-prediction-market-c2d81988f75e

use wasmlib::*;
use chrono::{DateTime,  Utc, NaiveDateTime};
use serde_with::serde_as;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

const IOTA_PER_GREEN_WH:i64 = 2500; // day price of collected renewable energy
const IOTA_PER_GREEN_WS:i64 = IOTA_PER_GREEN_WH/3600; // ≈ 0.70

const iota_per_standard_Wh:i64 = 3000; // day price of standard energy
const iota_per_standard_Ws:i64 = iota_per_standard_Wh/3600; // ≈ 0.83

const AUCTION_DURATION:i64 = 1; // auction duraction in seconds

static TOTAL_CURRENT: AtomicI64 = AtomicI64::new(0);
static TOTAL_PUSHED: AtomicI64 = AtomicI64::new(0);
static TOTAL_REQUESTED: AtomicI64 = AtomicI64::new(0);

pub fn get_total_current() -> i64 {
    TOTAL_CURRENT.load(Ordering::Relaxed)
}

pub fn set_total_current(level: i64) {
    TOTAL_CURRENT.store(level, Ordering::Relaxed);
}

pub fn get_total_pushed() -> i64 {
    TOTAL_PUSHED.load(Ordering::Relaxed)
}

pub fn set_total_pushed(level: i64) {
    TOTAL_PUSHED.store(level, Ordering::Relaxed);
}

pub fn get_total_requested() -> i64 {
    TOTAL_REQUESTED.load(Ordering::Relaxed)
}

pub fn set_total_requested(level: i64) {
    TOTAL_REQUESTED.store(level, Ordering::Relaxed);
}



#[no_mangle]
fn on_load() {
    // functions of the smart contract
    let exports = ScExports::new();
    exports.add_func("trade", trade );
    exports.add_func("initmarket", initmarket);
    exports.add_func("closemarket", closemarket);
}

// The contract owner should call this function for initialization and to set an end time for trading 
// using the parameter TRADEENDUTC, which is a date and time string in ISO format, assuming UTC.
fn initmarket(context: &ScFuncContext) {
    // only contract owner should be able to do this
    let creator = context.contract_creator();
    let caller = context.caller();
    context.require(creator == caller, "Not authorised to init market - only contract creator is allowed to do this.");

    let mut log:String = "INITMARKET is run:".to_string();   context.log(&log);
    
    // a flag, stating that the closemarket function was not (successfully) run yet
    context.state().get_string("marketclosed").set_value(&"false".to_string());

    if context.params().get_string(&"TRADEENDUTC".to_string()).value()==""  {
        // default: do not use end time for trades
        context.state().get_int64(&"tradeenddatetime".to_string()).set_value(0);

        log = "Do not use specific end time for trades".to_string();  context.log(&log);
    }
    else {
        // parse ISO datetime string, e.g. "2021-01-01 02:00" (in UTC) and convert to UNIX timestamp
        let tradeenddatetime:i64 = DateTime::<Utc>::from_utc(NaiveDateTime::parse_from_str(&context.params().get_string(&"TRADEENDUTC".to_string()).value(), "%Y-%m-%d %H:%M").expect("failed to execute"), Utc).timestamp();

        log = "Trade end timestamp (UTC): ".to_string() + &tradeenddatetime.to_string();     context.log(&log);

        // store state
        context.state().get_int64(&"tradeenddatetime".to_string()).set_value(tradeenddatetime);
    }

    set_total_pushed(0);
    set_total_requested(0);

}

#[serde_as]
#[derive(Deserialize, Serialize)]
struct Trade {
    // value for which the trade is valid, e.g., "request" or "push" regarding the type of the trade
    // "requested" or "pushed" energy
    request_or_push: String,
    //energy amount in Watt
    energyamount: i64,
    // trade size in IOTA
    currency: i64,
}

#[serde_as]
#[derive(Deserialize, Serialize)]
struct ContainerOfTrades {
    // map trading account's wallet address (string) to a Trade
    map: HashMap<String,Trade>,
}


// function to trade energy, pushed or requested as defined in TRADEVALUE
// the amount to trade is the amount of IOTA or amount of energy sent with the function call
// trades must be placed in time before the tradeenddatetime has passed set on initialization
fn trade(context: &ScFuncContext) {
    let currtime:i64 = context.timestamp();  // transaction timestamp?!
    let tradeenddatetime:i64 = context.state().get_int64(&"tradeenddatetime".to_string()).value();

    // either we don't use a fixed end time - or we check if the end time is not exceeded
    if tradeenddatetime==0 || (tradeenddatetime!=0 && currtime <= tradeenddatetime) {
        let mut log:String = "TRADE is placed:".to_string(); context.log(&log);
        
        // get outcome value on which the trade was placed
        let tradevalue = context.params().get_string(&"TRADEVALUE".to_string());
        // require parameter exists
        context.require(tradevalue.exists(), "trade value parameter not found");
        
        // how much WATT was sent with the transaction?
        let incoming_energy = context.params().get_string(&"WATT".to_string());
        log = "energy amount (WATT): ".to_string() + &incoming_energy.to_string();   context.log(&log);

        // how much IOTA were sent with the transaction?
        let incoming = context.incoming().balance(&ScColor::IOTA);
        log = "trade amount (IOTA): ".to_string() + &incoming.to_string();   context.log(&log);
        

        if tradevalue.to_string() == "pushed" {
            set_total_pushed(get_total_pushed() + incoming_energy.to_string().parse::<i64>().unwrap());
        }

        else if tradevalue.to_string() == "requested" {
            set_total_requested(get_total_requested() + incoming_energy.to_string().parse::<i64>().unwrap());
        }

        // get wallet address of trading account
        let caller = context.caller().address();
        // store the value the trade refers to, e.g., "push" - per trading account
        context.state().get_map(&caller.to_string()).get_string(&"tradevalue".to_string()).set_value(&tradevalue.to_string());
        
        // store all trades as jsonified hashmap in the state, which does not allow iterating over a map
        let containeroftradesjson = context.state().get_string(&"containeroftradesjson".to_string()).value();
        let mut containeroftrades : ContainerOfTrades;

        // already stored?
        if containeroftradesjson == "" {
            containeroftrades = ContainerOfTrades {
                map : HashMap::new()
            };
        }
        else {
            // de-serialize and re-create the struct from string
            containeroftrades = serde_json::from_str(&containeroftradesjson).expect("failed to get container of trades");
        }

        // create Trade struct and store in map under the trading account's (wallet) address
        let trade = Trade  {
            request_or_push: tradevalue.to_string(),
            energyamount: incoming_energy.to_string().parse::<i64>().unwrap(),
            currency: incoming.to_string().parse::<i64>().unwrap(),
        };
        containeroftrades.map.insert(caller.to_string(), trade);

        // serialize all trades to a json string
        let containeroftradesjson = serde_json::to_string(&containeroftrades).expect("failed to make json of container of trades");
        // store state as a string
        context.state().get_string(&"containeroftradesjson".to_string()).set_value(&containeroftradesjson);
    } else {
        let log:String = "trade was not provided on time".to_string();
        context.log(&log);
    }
}


// Function to close the trading market, to be called by the contract owner.
// The functions runs through the stored trades, determines how mush each client should receive, and sends the IOTA to the wallets of the clients.
fn closemarket(context: &ScFuncContext) {
    // only contract owner should be able to do this
    let creator = context.contract_creator();
    let caller = context.caller();
    context.require(creator == caller, "You are not authorised to close the trading market - only contract creator is allowed to close the market.");

    // only close market after end time for trades, specified on initalization
    let currtime: i64 = context.timestamp();
    let tradeenddatetime: i64 = context.state().get_int64(&"tradeenddatetime".to_string()).value();

    let mut log:String;

    // a flag to check whether the closemarket function was run
    let marketclosed: String = context.state().get_string("marketclosed").to_string();
    if marketclosed.eq(&"false".to_string()) {
        // either we don't use a fixed end time - or we check if the end time is exceeded
        if tradeenddatetime == 0 || (tradeenddatetime != 0 && currtime > tradeenddatetime) {
            log = "CLOSEMARKET is executed:".to_string(); context.log(&log);

            // set flag stating that the closemarket function was run
            context.state().get_string("marketclosed").set_value(&"true".to_string());

            // get all trades from global state
            // Note that the stat is not specific to a contract but to the whole chain on which it is deployed
            let containeroftradesjson = context.state().get_string(&"containeroftradesjson".to_string()).value();
            let containeroftrades: ContainerOfTrades;

            if containeroftradesjson != "" {
                // get trades from json
                containeroftrades = serde_json::from_str(&containeroftradesjson).expect("failed to fetch container of trades");
                // we require more than one trade
                if containeroftrades.map.keys().len() >= 1 {

                    let mut receiveamount:i64;
                    let mut recipientaddress:ScAddress;
                    // send coins to clients
                    for (traderaddress, trade) in &containeroftrades.map {
                        if trade.request_or_push.eq("pushed") {
                            log = traderaddress.to_string() + &" placed a trade on \"".to_string() + &trade.request_or_push.to_string() + &"\", which is a THE RECEIVE".to_string(); context.log(&log);
                            let mut percentage = 1;
                            if get_total_pushed() > get_total_requested() {percentage = get_total_requested() / get_total_pushed()};
                            receiveamount = (trade.energyamount * IOTA_PER_GREEN_WS * AUCTION_DURATION) * percentage;
                            recipientaddress = ScAddress::from_bytes(&*context.utility().base58_decode(&traderaddress.to_string()));
                            log = "transferring amount of IOTA to: ".to_string() +  &recipientaddress.to_string();  context.log(&log);
                            context.transfer_to_address( &recipientaddress, ScTransfers::new(&ScColor::IOTA, receiveamount))
                        }

                        if trade.request_or_push.eq("requested") {
                            log = traderaddress.to_string() + &" placed a trade on \"".to_string() + &trade.request_or_push.to_string() + &"\", which is a THE RECEIVE".to_string(); context.log(&log);
                            let mut percentage = 1;
                            if get_total_requested() > get_total_pushed() {percentage = get_total_pushed() / get_total_requested()};
                            receiveamount = (trade.currency - trade.energyamount * IOTA_PER_GREEN_WS * AUCTION_DURATION) * percentage;
                                recipientaddress = ScAddress::from_bytes(&*context.utility().base58_decode(&traderaddress.to_string()));
                                log = "transferring amount of IOTA to: ".to_string() +  &recipientaddress.to_string();  context.log(&log);
                                context.transfer_to_address( &recipientaddress, ScTransfers::new(&ScColor::IOTA, receiveamount))
                        }
                    }
                } else {
                    log  = "at least one trade is required".to_string(); context.log(&log);
                }
            } else {
                log  = "no trades stored".to_string(); context.log(&log);
            }
        } else {
            log  = "closing the market can be only done after the end time for placing trades has passed".to_string(); context.log(&log);
        }    
    } else {
        log  = "the trading market was already closed".to_string(); context.log(&log);
    }
    
}
