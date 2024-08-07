use std::collections::VecDeque;

use reqwest::{self, blocking::get};

#[derive(Debug, Default)]
pub struct NetworkStats {
    pub hashrate: VecDeque<(f64, f64)>,
    pub difficulty: f64,
    pub height: u64,
    pub reward: u8,
    pub reward_reduction: u8,
    pub price: f64,
}

#[derive(Debug, Default)]
pub struct PoolStats {
    pub hashrate: VecDeque<(f64, f64)>,
    pub connected_miners: u64,
    pub effort: f64,
    pub total_blocks: u64,
    pub confirming_new_block: f64,
}

#[derive(Debug, Default)]
pub struct MinerStats {
    pub hashrate: VecDeque<(f64, f64)>,
    pub average_hashrate: f64,
    pub pending_shares: f64,
    pub pending_balance: f64,
    pub round_contribution: f64,
    pub total_paid: f64,
}

#[derive(Debug, Default)]
pub struct Stats {
    pub network: NetworkStats,
    pub pool: PoolStats,
    pub miner: MinerStats,
}

impl Stats {
    pub fn default() -> Stats {
        Stats {
            network: NetworkStats::default(),
            pool: PoolStats::default(),
            miner: MinerStats::default(),
        }
    }

    /// Get data from Mining Core API
    pub fn get_data(&mut self) -> Result<(), reqwest::Error> {
        let pool_api_url = "http://15.204.211.130:4000/api/pools/ErgoSigmanauts";
        let price_api_url = "https://api.spectrum.fi/v1/price-tracking/cmc/markets";
        let hashrate_api = "https://api.ergoplatform.com/info";
        let data: serde_json::Value = get(pool_api_url)?.json()?;

        //Format block height
        let block_height = data["pool"]["networkStats"]["blockHeight"].clone().as_u64();

        //Only update the data if a new block is added to the chain
        if block_height.unwrap() != self.network.height {
            match block_height {
                Some(block_height) => self.network.height = block_height,
                None => println!("No data available for Block Height"),
            }

            let price_data: serde_json::Value = get(price_api_url)?.json()?;
            let hashrate_data: serde_json::Value = get(hashrate_api)?.json()?;

            // Network Hashrate
            let network_hashrate = hashrate_data["hashRate"].clone().as_f64();

            match network_hashrate {
                Some(network_hashrate) => {
                    let network_hashrate =
                        ((network_hashrate / 1_000_000_000_000.0) * 100.0).round() / 100.0;
                    self.network
                        .hashrate
                        .push_back((block_height.unwrap() as f64, network_hashrate));
                }

                None => println!("No data available for Network Hashrate"),
            }

            // Network Difficulty
            let network_difficulty = data["pool"]["networkStats"]["networkDifficulty"]
                .clone()
                .as_f64();

            match network_difficulty {
                Some(network_difficulty) => {
                    //round to 2 decimals
                    let network_difficulty =
                        ((network_difficulty / 1_000_000_000_000_000.0) * 100.0).round() / 100.0;
                    self.network.difficulty = network_difficulty;
                }

                None => println!("No data available for Network Difficulty"),
            }

            // ERG Price

            if let serde_json::Value::Array(arr) = price_data {
                for obj in arr {
                    if let serde_json::Value::Object(o) = obj {
                        if let Some(base_name) = o.get("base_name") {
                            if base_name == "ERG" {
                                if let Some(quote_name) = o.get("quote_name") {
                                    if quote_name == "SigUSD" {
                                        if let Some(last_price) = o.get("last_price") {
                                            if let Some(price) = last_price.as_f64() {
                                                self.network.price =
                                                    ((1.0 / price) * 100.0).round() / 100.0;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            //Pool hashrate
            let pool_hashrate = data["pool"]["poolStats"]["poolHashrate"].clone().as_f64();

            match pool_hashrate {
                Some(pool_hashrate) => {
                    let pool_hashrate = ((pool_hashrate / 1_000_000_000.0) * 100.0).round() / 100.0;
                    self.pool
                        .hashrate
                        .push_back((self.network.height as f64, pool_hashrate))
                }

                None => println!("No data available for Pool Hashrate"),
            }

            //Pool connected miners
            let connected_miners = data["pool"]["poolStats"]["connectedMiners"]
                .clone()
                .as_u64();

            match connected_miners {
                Some(connected_miners) => self.pool.connected_miners = connected_miners,

                None => println!("No data available for Connected Miners"),
            }

            //Pool effort
            let pool_effort = data["pool"]["poolEffort"].clone().as_f64();

            match pool_effort {
                Some(pool_effort) => {
                    let pool_effort = (pool_effort * 10000.0).round() / 100.0;
                    self.pool.effort = pool_effort;
                }

                None => println!("No data available for Pool Effort"),
            }

            //Pool total blocks
            let pool_total_blocks = data["pool"]["totalBlocks"].clone().as_u64();

            match pool_total_blocks {
                Some(pool_total_blocks) => {
                    self.pool.total_blocks = pool_total_blocks;
                }

                None => println!("No data available for Pool Effort"),
            }

            //Pool confirming new block

            let block_data: serde_json::Value = get(format!("{}/blocks", pool_api_url))?.json()?;

            let pool_block_confirmation: (&str, f64) = (
                block_data[0]["status"].as_str().unwrap(),
                (block_data[0]["confirmationProgress"].as_f64().unwrap()) * 100.0,
            );

            if pool_block_confirmation.0 == "pending" {
                self.pool.confirming_new_block = pool_block_confirmation.1;
            } else {
                self.pool.confirming_new_block = 100.0;
            }
        }

        //Store only the last 720 blocks (720 * 2min = 24h)
        if self.network.hashrate.len() > 720 {
            self.network.hashrate.pop_front();
            self.pool.hashrate.pop_front();
        }

        Ok(())
    }
}
