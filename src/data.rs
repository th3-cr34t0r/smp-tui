use rand::*;
use reqwest::{self, blocking::get};

#[derive(Debug, Default)]
pub struct NetworkStats {
    pub hashrate: Vec<(f64, f64)>,
    pub difficulty: f64,
    pub height: u64,
    pub reward: u8,
    pub reward_reduction: u8,
    pub price: u8,
}

#[derive(Debug, Default)]
pub struct PoolStats {
    pub hashrate: Vec<(f64, f64)>,
    pub connected_miners: u64,
    pub effort: f64,
    pub total_blocks: u64,
    pub block_found_time: u8,
}

#[derive(Debug, Default)]
pub struct MinerStats {
    pub hashrate: Vec<(f64, f64)>,
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
        let url = "http://15.204.211.130:4000/api/pools/ErgoSigmanauts";

        let data: serde_json::Value = get(url)?.json()?;

        //Format block height
        let block_height = data["pool"]["networkStats"]["blockHeight"].clone().as_u64();

        //Only update the data if a new block is added to the chain
        if block_height.unwrap() != self.network.height {
            match block_height {
                Some(block_height) => self.network.height = block_height,
                None => println!("No data available for Block Height"),
            }

            // Network Hashrate
            let network_hashrate = data["pool"]["networkStats"]["networkHashrate"]
                .clone()
                .as_f64();

            match network_hashrate {
                Some(network_hashrate) => {
                    let network_hashrate =
                        ((network_hashrate / 1_000_000_000_000.0) * 100.0).round() / 100.0;
                    self.network.hashrate.append(&mut vec![(
                        block_height.unwrap() as f64 + 1.0,
                        network_hashrate,
                    )]);
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

            //Pool hashrate
            let pool_hashrate = data["pool"]["poolStats"]["poolHashrate"].clone().as_f64();

            match pool_hashrate {
                Some(pool_hashrate) => {
                    let pool_hashrate = ((pool_hashrate / 1_000_000_000.0) * 100.0).round() / 100.0;
                    self.pool
                        .hashrate
                        .append(&mut vec![(self.network.height as f64 + 1.0, pool_hashrate)])
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
        }
        self.miner.hashrate = self.get_hashrate();

        if self.network.hashrate.len() > 100 {
            self.network.hashrate.remove(0);
            self.pool.hashrate.remove(0);
        }

        Ok(())
    }

    fn get_hashrate(&self) -> Vec<(f64, f64)> {
        let mut data: Vec<(f64, f64)> = vec![];
        let mut rng = rand::thread_rng();
        for i in 0..10 {
            data.insert(i, ((i + 1) as f64, rng.gen_range(15..=20) as f64));
        }
        data
    }
}
