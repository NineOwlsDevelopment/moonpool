import dotenv from "dotenv";
dotenv.config();
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Moonpool } from "../target/types/moonpool";
import { Connection } from "@solana/web3.js";
import { PublicKey, Keypair } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";

const generateTokenAccount = async (asset: Keypair) => {
  return await getOrCreateAssociatedTokenAccount(
    connection,
    payer.payer,
    asset.publicKey,
    payer.publicKey,
    false
  );
};

export const RPC_URL = process.env.HELIUS_RPC_URL || "http://127.0.0.1:8899";
export const DECIMALS_PER_TOKEN = 1_000_000;
export const NATIVE_MINT = new PublicKey(
  "So11111111111111111111111111111111111111112"
);
export const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
export const CP_SWAP_PROGRAM = new PublicKey(
  "CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW"
);
export const CONFIG_ADDRESS = new PublicKey(
  "9zSzfkYy6awexsHvmggeH36pfVUdDGyCcwmjT3AQPBj6"
);
export const CREATE_POOL_FEE_RECEIVE = new PublicKey(
  "G11FKBRaAkHAKuLCgLM6K6NUc9rTjPAznRCjZifrTQe2"
);

export const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

export const payer = provider.wallet as anchor.Wallet;
export const connection = new Connection(RPC_URL, "confirmed");
export const program = anchor.workspace.Moonpool as Program<Moonpool>;

export const [MOONPOOL_PDA] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("moonpool")],
  program.programId
);

export const [FEE_VAULT_PDA] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("fee_vault")],
  program.programId
);
