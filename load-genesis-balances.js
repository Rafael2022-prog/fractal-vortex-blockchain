const axios = require('axios');
const fs = require('fs');

// Load genesis configuration
const genesisFile = 'mainnet-genesis-final.json';

async function loadGenesisBalances() {
    console.log('🚀 Loading Genesis Balances to LedgerDB');
    console.log('═══════════════════════════════════════════════════════════════');
    
    try {
        // Read genesis file
        if (!fs.existsSync(genesisFile)) {
            console.error(`❌ Genesis file not found: ${genesisFile}`);
            return;
        }
        
        const genesisData = JSON.parse(fs.readFileSync(genesisFile, 'utf8'));
        const allocations = genesisData.alloc;
        
        if (!allocations) {
            console.error('❌ No allocations found in genesis file');
            return;
        }
        
        console.log(`\n📋 Found ${Object.keys(allocations).length} wallet allocations in genesis`);
        
        // Convert hex addresses to FVC format and load balances
        for (const [hexAddress, allocation] of Object.entries(allocations)) {
            const balance = allocation.balance;
            const description = allocation.description || 'Genesis allocation';
            
            // Convert hex address to FVC format
            const fvcAddress = convertHexToFvcAddress(hexAddress);
            
            console.log(`\n💰 Loading balance for: ${description}`);
            console.log(`   Hex Address: 0x${hexAddress}`);
            console.log(`   FVC Address: ${fvcAddress}`);
            console.log(`   Balance: ${formatFvcAmount(balance)} FVC`);
            
            try {
                // Use the admin endpoint to set balance directly
                const response = await axios.post('http://localhost:8080/admin/set-balance', {
                    address: fvcAddress,
                    balance: balance
                });
                
                if (response.data.success) {
                    console.log(`   ✅ Balance loaded successfully`);
                } else {
                    console.log(`   ❌ Failed to load balance: ${response.data.error || 'Unknown error'}`);
                }
            } catch (error) {
                // If admin endpoint doesn't exist, try alternative method
                console.log(`   ⚠️  Admin endpoint not available, trying alternative method...`);
                
                // Try to use mining to generate balance (for genesis wallet only)
                if (description.includes('Genesis') || description.includes('Transaction Fee')) {
                    await generateBalanceViaMining(fvcAddress, balance);
                } else {
                    console.log(`   ❌ Cannot load balance for non-genesis wallet without admin endpoint`);
                }
            }
        }
        
        console.log('\n🔍 Verifying loaded balances...');
        await verifyLoadedBalances(allocations);
        
    } catch (error) {
        console.error(`❌ Error loading genesis balances: ${error.message}`);
    }
}

function convertHexToFvcAddress(hexAddress) {
    // Remove 0x prefix and convert to FVC format
    const cleanHex = hexAddress.replace('0x', '');
    return `fvc${cleanHex}`;
}

function formatFvcAmount(amountStr) {
    const amount = BigInt(amountStr);
    const fvcAmount = Number(amount) / 1_000_000_000; // 9 decimal places
    return fvcAmount.toLocaleString();
}

async function generateBalanceViaMining(address, targetBalance) {
    console.log(`   ⛏️  Attempting to generate balance via mining...`);
    
    try {
        // Start mining for this address
        const startResponse = await axios.post('http://localhost:8080/miner/start', {
            address: address,
            device_id: `genesis_${address.slice(-8)}`
        });
        
        if (startResponse.data.success) {
            console.log(`   ✅ Mining started for ${address}`);
            
            // Wait for some mining rewards
            let attempts = 0;
            const maxAttempts = 10;
            
            while (attempts < maxAttempts) {
                await new Promise(resolve => setTimeout(resolve, 5000)); // Wait 5 seconds
                
                const balanceResponse = await axios.get('http://localhost:8080/wallet/balance', {
                    params: { address: address }
                });
                
                const currentBalance = balanceResponse.data.balance || 0;
                console.log(`   💰 Current Balance: ${formatFvcAmount(currentBalance.toString())} FVC`);
                
                if (currentBalance > 0) {
                    console.log(`   ✅ Mining generated some balance`);
                    break;
                }
                
                attempts++;
            }
            
            // Stop mining
            await axios.post('http://localhost:8080/miner/stop', {
                device_id: `genesis_${address.slice(-8)}`
            });
            
        } else {
            console.log(`   ❌ Failed to start mining: ${startResponse.data.error || 'Unknown error'}`);
        }
    } catch (error) {
        console.log(`   ❌ Mining error: ${error.message}`);
    }
}

async function verifyLoadedBalances(allocations) {
    for (const [hexAddress, allocation] of Object.entries(allocations)) {
        const fvcAddress = convertHexToFvcAddress(hexAddress);
        const expectedBalance = allocation.balance;
        
        try {
            const response = await axios.get('http://localhost:8080/wallet/balance', {
                params: { address: fvcAddress }
            });
            
            const actualBalance = response.data.balance || 0;
            const description = allocation.description || 'Genesis allocation';
            
            console.log(`\n🔍 ${description}:`);
            console.log(`   Address: ${fvcAddress}`);
            console.log(`   Expected: ${formatFvcAmount(expectedBalance)} FVC`);
            console.log(`   Actual: ${formatFvcAmount(actualBalance.toString())} FVC`);
            
            if (actualBalance.toString() === expectedBalance) {
                console.log(`   ✅ Balance matches`);
            } else {
                console.log(`   ❌ Balance mismatch`);
            }
        } catch (error) {
            console.log(`   ❌ Error checking balance: ${error.message}`);
        }
    }
}

async function main() {
    await loadGenesisBalances();
    
    console.log('\n✅ Genesis balance loading completed!');
    console.log('🔍 Check the dashboard to verify wallet balances.');
}

main().catch(console.error);