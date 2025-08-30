// FVChain API Service Example
// Copy this file to fvchainApi.ts and update with your actual values

export interface NetworkInfo {
  chainId: number;
  networkName: string;
  blockHeight: number;
  totalSupply: string;
  circulatingSupply: string;
  difficulty: number;
  hashRate: string;
  peers: number;
}

export interface WalletBalance {
  address: string;
  balance: string;
  nonce: number;
}

export interface Transaction {
  hash: string;
  from: string;
  to: string;
  amount: string;
  fee: string;
  timestamp: number;
  blockHeight: number;
  status: 'pending' | 'confirmed' | 'failed';
}

export interface CreateWalletResponse {
  address: string;
  privateKey: string;
  publicKey: string;
  mnemonic: string;
}

export interface SendTransactionRequest {
  from: string;
  to: string;
  amount: string;
  privateKey: string;
  fee?: string;
}

export interface SendTransactionResponse {
  hash: string;
  status: string;
}

export class FVChainApiService {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }

  async getNetworkInfo(): Promise<NetworkInfo> {
    const response = await fetch(`${this.baseUrl}/network/info`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async getWalletBalance(address: string): Promise<WalletBalance> {
    const response = await fetch(`${this.baseUrl}/wallet/balance/${address}`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async createWallet(): Promise<CreateWalletResponse> {
    const response = await fetch(`${this.baseUrl}/wallet/create`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async importWallet(privateKey: string): Promise<{ address: string; publicKey: string }> {
    const response = await fetch(`${this.baseUrl}/wallet/import`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ privateKey }),
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async sendTransaction(request: SendTransactionRequest): Promise<SendTransactionResponse> {
    const response = await fetch(`${this.baseUrl}/transactions/send`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async getTransactionHistory(address: string, limit: number = 10): Promise<Transaction[]> {
    const response = await fetch(`${this.baseUrl}/transactions/history/${address}?limit=${limit}`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async getTransaction(hash: string): Promise<Transaction> {
    const response = await fetch(`${this.baseUrl}/transactions/${hash}`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async getBlocks(limit: number = 10): Promise<any[]> {
    const response = await fetch(`${this.baseUrl}/blocks?limit=${limit}`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async getBlock(height: number): Promise<any> {
    const response = await fetch(`${this.baseUrl}/blocks/${height}`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async healthCheck(): Promise<{ status: string; timestamp: number }> {
    const response = await fetch(`${this.baseUrl}/network/health`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }
}

export default FVChainApiService;