#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contractmeta, panic_with_error,
    symbol_short, vec, Address, Env, Symbol, Vec,
};

contractmeta!(key = "title", val = "KaleFail Tractor",);
contractmeta!(
    key = "desc",
    val = "Harvest all available KALE for your farmer.",
);
contractmeta!(key = "binver", val = "2.0.0",);

const FARM: Symbol = symbol_short!("FARM");
const DAY_OF_LEDGERS: u32 = 60 * 60 * 24 / 5; // assumes 5-second ledger close times
const WEEK_OF_LEDGERS: u32 = DAY_OF_LEDGERS * 7;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// No pails provided in invocation
    NoPailsProvided = 1,
    /// Harvesting all pails results in 0 reward
    NoHarvestablePails = 2,
}

#[contractclient(name = "FarmClient")]
pub trait Farm {
    fn harvest(env: Env, farmer: Address, index: u32) -> i128;
}

pub fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .instance()
        .extend_ttl(max_ttl - WEEK_OF_LEDGERS, max_ttl);
}

#[contract]
pub struct KaleTractorContract;

#[contractimpl]
impl KaleTractorContract {
    pub fn __constructor(env: Env, farm: Address) {
        env.storage().instance().set(&FARM, &farm);
        extend_instance_ttl(&env);
    }

    /// Harvest multiple pails available for your KALE farmer.
    ///
    /// # Arguments
    /// - `farmer` - address of the farmer to harvest on behalf of
    /// - `pails` - vector of pails which should be harvested
    ///
    /// # Panics
    /// - If the `pails` vector is empty
    /// - If no pails result in a non-zero reward
    pub fn harvest(env: Env, farmer: Address, pails: Vec<u32>) -> Vec<i128> {
        if pails.len() == 0 {
            panic_with_error!(&env, Error::NoPailsProvided);
        }

        let mut rewards: Vec<i128> = vec![&env];
        let mut at_least_one_nonzero: bool = false;

        let farm_address: Address = env.storage().instance().get(&FARM).unwrap();
        let farm_client = FarmClient::new(&env, &farm_address);

        // iterate through pails provided
        pails.iter().for_each(|pail| {
            // try to invoke the farm's `harvest` function
            let reward = match farm_client.try_harvest(&farmer, &pail) {
                Ok(Ok(number)) => number, // successful harvest, use the reward
                _ => 0,                   // unsuccessful for some reason, use 0
            };
            rewards.push_back(reward);

            if !at_least_one_nonzero && reward > 0 {
                at_least_one_nonzero = true;
            }
        });

        // make sure we've at least harvested _some_ kale
        if !at_least_one_nonzero {
            panic_with_error!(&env, Error::NoHarvestablePails);
        };

        extend_instance_ttl(&env);

        return rewards;
    }
}

mod test;
