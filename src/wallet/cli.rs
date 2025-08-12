use clap::{Parser, Subcommand};
use crate::wallet::Wallet;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fvc-wallet")]
#[command(about = "Fractal-Vortex Chain Official Wallet CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long, default_value = "http://localhost:8080")]
    rpc_url: String,
    
    #[arg(long, short)]
    wallet_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new wallet
    Create {
        #[arg(long)]
        save_to: Option<PathBuf>,
        
        #[arg(long)]
        mnemonic: Option<String>,
    },
    
    /// Show wallet information
    Info {
        #[arg(long)]
        address: bool,
        
        #[arg(long)]
        balance: bool,
    },
    
    /// Send FVC to another address
    Send {
        to: String,
        amount: u64,
        
        #[arg(long)]
        fee: Option<u64>,
    },
    
    /// Stake FVC to a validator
    Stake {
        validator: String,
        amount: u64,
    },
    
    /// Unstake FVC from a validator
    Unstake {
        validator: String,
        amount: u64,
    },
    
    /// Show staking information
    Staking {
        #[arg(long)]
        validator: Option<String>,
    },
    
    /// Show transaction status
    Tx {
        hash: String,
    },
    
    /// Show network information
    Network,
    
    /// Show recent transactions
    History {
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    
    /// Export wallet to file
    Export {
        path: PathBuf,
    },
    
    /// Import wallet from file
    Import {
        path: PathBuf,
    },
}

impl Cli {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::Create { save_to, mnemonic } => {
                let wallet = if let Some(mnemonic) = mnemonic {
                    Wallet::from_mnemonic(mnemonic, &self.rpc_url)
                } else {
                    Wallet::new(&self.rpc_url)
                };
                
                println!("Wallet created successfully!");
                println!("Address: {}", wallet.get_address());
                println!("Peer ID: {}", wallet.get_peer_id());
                
                if let Some(path) = save_to {
                    wallet.save_to_file(path.to_str().unwrap())?;
                    println!("Wallet saved to: {:?}", path);
                }
                
                if mnemonic.is_none() {
                    let mnemonic = Wallet::generate_mnemonic();
                    println!("Mnemonic: {}", mnemonic);
                    println!("⚠️  Save this mnemonic securely!");
                }
            }
            
            Commands::Info { address, balance } => {
                let wallet = self.load_wallet().await?;
                
                if *address {
                    println!("Address: {}", wallet.get_address());
                    println!("Peer ID: {}", wallet.get_peer_id());
                }
                
                if *balance {
                    let balance = wallet.get_balance().await?;
                    println!("Balance: {} FVC", balance);
                }
                
                if !*address && !*balance {
                    println!("Address: {}", wallet.get_address());
                    println!("Peer ID: {}", wallet.get_peer_id());
                    let balance = wallet.get_balance().await?;
                    println!("Balance: {} FVC", balance);
                }
            }
            
            Commands::Send { to, amount, fee } => {
                let mut wallet = self.load_wallet().await?;
                
                let estimated_fee = wallet.estimate_fee("transfer", *amount).await?;
                let actual_fee = fee.unwrap_or(estimated_fee);
                
                println!("Sending {} FVC to {}", amount, to);
                println!("Fee: {} FVC", actual_fee);
                
                let tx_hash = wallet.transfer(to, *amount).await?;
                println!("Transaction sent! Hash: {}", tx_hash);
            }
            
            Commands::Stake { validator, amount } => {
                let mut wallet = self.load_wallet().await?;
                
                let estimated_fee = wallet.estimate_fee("stake", *amount).await?;
                println!("Staking {} FVC to validator {}", amount, validator);
                println!("Estimated fee: {} FVC", estimated_fee);
                
                let tx_hash = wallet.stake(validator, *amount).await?;
                println!("Stake transaction sent! Hash: {}", tx_hash);
            }
            
            Commands::Unstake { validator, amount } => {
                let mut wallet = self.load_wallet().await?;
                
                let estimated_fee = wallet.estimate_fee("unstake", *amount).await?;
                println!("Unstaking {} FVC from validator {}", amount, validator);
                println!("Estimated fee: {} FVC", estimated_fee);
                
                let tx_hash = wallet.unstake(validator, *amount).await?;
                println!("Unstake transaction sent! Hash: {}", tx_hash);
            }
            
            Commands::Staking { validator } => {
                let wallet = self.load_wallet().await?;
                
                let staking_info = wallet.get_staking_info().await?;
                let validators = wallet.get_validators().await?;
                
                println!("Available validators: {:?}", validators);
                println!("Your staking info: {:?}", staking_info);
                
                if let Some(validator_addr) = validator {
                    let amount = staking_info.get(validator_addr).unwrap_or(&0);
                    println!("Staked to {}: {} FVC", validator_addr, amount);
                }
            }
            
            Commands::Tx { hash } => {
                let wallet = self.load_wallet().await?;
                
                let status = wallet.get_transaction_status(hash).await?;
                println!("Transaction status: {:?}", status);
            }
            
            Commands::Network => {
                let wallet = self.load_wallet().await?;
                
                let network_info = wallet.get_network_info().await?;
                println!("Network Information:");
                println!("Chain ID: {}", network_info.chain_id);
                println!("Block Height: {}", network_info.block_height);
                println!("Network Hash Rate: {:.2} H/s", network_info.network_hash_rate);
                println!("Active Validators: {}", network_info.active_validators);
                println!("Total Supply: {} FVC", network_info.total_supply);
            }
            
            Commands::History { limit } => {
                let wallet = self.load_wallet().await?;
                
                let transactions = wallet.get_recent_transactions(*limit).await?;
                println!("Recent transactions:");
                
                for tx in transactions {
                    println!("Hash: {}", tx.hash.as_ref().unwrap_or(&"pending".to_string()));
                    println!("From: {} -> To: {}", tx.from, tx.to);
                    println!("Amount: {} FVC", tx.amount);
                    println!("Fee: {} FVC", tx.fee);
                    println!("---");
                }
            }
            
            Commands::Export { path } => {
                let wallet = self.load_wallet().await?;
                wallet.save_to_file(path.to_str().unwrap())?;
                println!("Wallet exported to: {:?}", path);
            }
            
            Commands::Import { path } => {
                let wallet = Wallet::from_file(path.to_str().unwrap(), &self.rpc_url)?;
                
                if let Some(save_path) = &self.wallet_file {
                    wallet.save_to_file(save_path.to_str().unwrap())?;
                }
                
                println!("Wallet imported successfully!");
                println!("Address: {}", wallet.get_address());
                println!("Balance: {} FVC", wallet.get_balance().await?);
            }
        }
        
        Ok(())
    }
    
    async fn load_wallet(&self) -> Result<Wallet, Box<dyn std::error::Error>> {
        let wallet_file = self.wallet_file.as_ref()
            .ok_or("Wallet file not specified. Use --wallet-file or create a new wallet")?;
        
        Wallet::from_file(wallet_file.to_str().unwrap(), &self.rpc_url)
    }
}

pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    cli.run().await
}