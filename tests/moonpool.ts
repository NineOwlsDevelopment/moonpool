import { BN } from "@coral-xyz/anchor";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { CpmmPoolInfoLayout } from "@raydium-io/raydium-sdk-v2";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  ComputeBudgetProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createSyncNativeInstruction,
} from "@solana/spl-token";
import {
  program,
  MOONPOOL_PDA,
  payer,
  connection,
  TOKEN_METADATA_PROGRAM_ID,
  FEE_VAULT_PDA,
  CP_SWAP_PROGRAM,
  CONFIG_ADDRESS,
  CREATE_POOL_FEE_RECEIVE,
  NATIVE_MINT,
} from "./constants";
import { uploadToIPFS } from "./helpers";
import { expect } from "chai";

let ASSET_A: Keypair;
let ASSET_B: Keypair;
let ASSET_C: Keypair;

let ASSET_TOKEN_ACCOUNT_A: PublicKey;
let ASSET_TOKEN_ACCOUNT_B: PublicKey;
let ASSET_TOKEN_ACCOUNT_C: PublicKey;

const POOL_NAME = Math.random().toString(36).substring(2, 8);

const isInitialized = true;

describe("moonpool", () => {
  beforeEach(function () {
    this.timeout(70000);
  });

  it("Initializes the program", async () => {
    if (isInitialized) {
      return;
    }

    await program.methods
      .initialize()
      .accounts({
        payer: payer.publicKey,
        moonpool: MOONPOOL_PDA,
        feeVault: FEE_VAULT_PDA,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
      .then((tx) => {
        console.log(tx);
      })
      .catch((err) => {
        throw err;
      });
  });

  it("Creates a pool and a pool mint", async () => {
    const [POOL_PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), payer.publicKey.toBuffer(), Buffer.from(POOL_NAME)],
      program.programId
    );

    const [DROPLET_MINT] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint"), POOL_PDA.toBuffer()],
      program.programId
    );

    const [METADATA_PDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        DROPLET_MINT.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const [POOL_WSOL_VAULT] = PublicKey.findProgramAddressSync(
      [Buffer.from("wsol_vault"), POOL_PDA.toBuffer()],
      program.programId
    );

    const [POOL_DROPLET_VAULT] = PublicKey.findProgramAddressSync(
      [Buffer.from("droplet_vault"), POOL_PDA.toBuffer()],
      program.programId
    );

    const symbol = "TEST";
    const metadataUri = await uploadToIPFS(
      POOL_NAME,
      symbol,
      DROPLET_MINT.toBase58()
    );
    const raiseGoal = 0.5 * LAMPORTS_PER_SOL;

    const createPoolInstruction: TransactionInstruction = await program.methods
      .createPool(POOL_NAME, symbol, new BN(raiseGoal))
      .accounts({
        moonpool: MOONPOOL_PDA,
        pool: POOL_PDA,
        feeVault: FEE_VAULT_PDA,
        poolWsolVault: POOL_WSOL_VAULT,
        payer: payer.publicKey,
        wsolMint: NATIVE_MINT,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const createPoolMintInstruction: TransactionInstruction =
      await program.methods
        .createPoolMint(metadataUri)
        .accounts({
          moonpool: MOONPOOL_PDA,
          pool: POOL_PDA,
          poolWsolVault: POOL_WSOL_VAULT,
          poolDropletVault: POOL_DROPLET_VAULT,
          dropletMint: DROPLET_MINT,
          metadata: METADATA_PDA,
          payer: payer.publicKey,
          wsolMint: NATIVE_MINT,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .instruction();

    const transaction = new Transaction()
      .add(createPoolInstruction)
      .add(createPoolMintInstruction);

    const tx = await sendAndConfirmTransaction(connection, transaction, [
      payer.payer,
    ]);

    // get pool account
    const pool = await program.account.pool.fetch(POOL_PDA);
    console.log(pool);
    console.log("raise goal", pool.raiseGoal.toNumber() / LAMPORTS_PER_SOL);
    console.log("maturity date", pool.maturityDate);
  });

  it("Contributes to the pool", async () => {
    const [POOL_PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), payer.publicKey.toBuffer(), Buffer.from(POOL_NAME)],
      program.programId
    );

    const [DROPLET_MINT] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint"), POOL_PDA.toBuffer()],
      program.programId
    );

    const [FEE_VAULT_PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("fee_vault")],
      program.programId
    );

    const [POOL_WSOL_VAULT] = PublicKey.findProgramAddressSync(
      [Buffer.from("wsol_vault"), POOL_PDA.toBuffer()],
      program.programId
    );

    const [POOL_DROPLET_VAULT] = PublicKey.findProgramAddressSync(
      [Buffer.from("droplet_vault"), POOL_PDA.toBuffer()],
      program.programId
    );

    const payerWsolTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      NATIVE_MINT,
      payer.publicKey,
      false
    );

    const payerDropletTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      DROPLET_MINT,
      payer.publicKey,
      false
    );

    const sol_amount_to_contribute = 0.5;
    const droplets_per_sol = 1_000_000_000 / 0.5; // 300 is the raise goal
    const droplets_to_mint = sol_amount_to_contribute * droplets_per_sol;
    console.log(Math.floor(droplets_to_mint * Math.pow(10, 6)));

    await program.methods
      .contribute(new BN(sol_amount_to_contribute * LAMPORTS_PER_SOL))
      .accounts({
        moonpool: MOONPOOL_PDA,
        feeVault: FEE_VAULT_PDA,
        pool: POOL_PDA,
        poolOwner: payer.publicKey,
        poolWsolVault: POOL_WSOL_VAULT,
        poolDropletVault: POOL_DROPLET_VAULT,
        payerWsolTokenAccount: payerWsolTokenAccount.address,
        payerDropletTokenAccount: payerDropletTokenAccount.address,
        payer: payer.publicKey,
        dropletMint: DROPLET_MINT,
        wsolMint: NATIVE_MINT,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
      .then((tx) => {
        console.log(tx);
      })
      .catch((err) => {
        console.log(err);
        throw err;
      });

    const pool = await program.account.pool.fetch(POOL_PDA);
    console.log(pool);
    console.log("raise goal", pool.raiseGoal.toNumber());
    console.log("droplet supply", pool.dropletSupply.toNumber());
    console.log("total raised", pool.totalRaised.toNumber());
  });

  return;

  it("Creates token mints and mints tokens to the payer", async () => {
    const mint1 = Keypair.generate();
    const mint2 = Keypair.generate();

    // Compare public keys and assign accordingly
    if (mint1.publicKey.toBuffer().compare(mint2.publicKey.toBuffer()) < 0) {
      ASSET_A = mint1;
      ASSET_B = mint2;
    } else {
      ASSET_A = mint2;
      ASSET_B = mint1;
    }

    // create a mint for asset A
    await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      6,
      ASSET_A
    );

    let myTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      ASSET_A.publicKey,
      payer.publicKey,
      false
    );
    ASSET_TOKEN_ACCOUNT_A = myTokenAccount.address;

    await mintTo(
      connection,
      payer.payer,
      ASSET_A.publicKey,
      ASSET_TOKEN_ACCOUNT_A,
      payer.publicKey,
      100000 * 10 ** 6
    );

    // create another mint for asset B
    await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      6,
      ASSET_B
    );

    myTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      ASSET_B.publicKey,
      payer.publicKey,
      false
    );
    ASSET_TOKEN_ACCOUNT_B = myTokenAccount.address;

    await mintTo(
      connection,
      payer.payer,
      ASSET_B.publicKey,
      ASSET_TOKEN_ACCOUNT_B,
      payer.publicKey,
      100000 * 10 ** 6
    );

    // wait 30 seconds
    await new Promise((resolve) => setTimeout(resolve, 30000));
  });

  // it("Initializes Raydium MockSOL-MockToken LP", async () => {
  //   const [CP_SWAP_AUTHORITY] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("vault_and_lp_mint_auth_seed")],
  //     CP_SWAP_PROGRAM
  //   );

  //   // For SOL-Token pool, NATIVE_MINT (SOL) should be token0
  //   const [POOL_STATE] = PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from("pool"),
  //       CONFIG_ADDRESS.toBuffer(),
  //       NATIVE_MINT.toBuffer(),
  //       ASSET_B.publicKey.toBuffer(), // Your token mint
  //     ],
  //     CP_SWAP_PROGRAM
  //   );

  //   const [LP_MINT] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("pool_lp_mint"), POOL_STATE.toBuffer()],
  //     CP_SWAP_PROGRAM
  //   );

  //   const [CREATOR_LP_TOKEN_ADDRESS] = PublicKey.findProgramAddressSync(
  //     [
  //       payer.publicKey.toBuffer(),
  //       TOKEN_PROGRAM_ID.toBuffer(),
  //       LP_MINT.toBuffer(),
  //     ],
  //     ASSOCIATED_PROGRAM_ID
  //   );

  //   const [VAULT_0] = PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from("pool_vault"),
  //       POOL_STATE.toBuffer(),
  //       NATIVE_MINT.toBuffer(),
  //     ],
  //     CP_SWAP_PROGRAM
  //   );

  //   const [VAULT_1] = PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from("pool_vault"),
  //       POOL_STATE.toBuffer(),
  //       ASSET_B.publicKey.toBuffer(),
  //     ],
  //     CP_SWAP_PROGRAM
  //   );

  //   const [OBSERVATION_STATE] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("observation"), POOL_STATE.toBuffer()],
  //     CP_SWAP_PROGRAM
  //   );

  //   const creatorToken0Account = await getOrCreateAssociatedTokenAccount(
  //     connection,
  //     payer.payer,
  //     NATIVE_MINT,
  //     payer.publicKey,
  //     false
  //   );

  //   const creatorToken1Account = await getOrCreateAssociatedTokenAccount(
  //     connection,
  //     payer.payer,
  //     ASSET_B.publicKey,
  //     payer.publicKey,
  //     false
  //   );

  //   // const initAmount0 = new BN(1000).mul(new BN(10 ** 6)); // MockSOL amount
  //   const initAmount0 = new BN(1).mul(new BN(10 ** 9));
  //   const initAmount1 = new BN(1000).mul(new BN(10 ** 6)); // MockToken amount
  //   const openTime = new BN(0);

  //   let tx = new Transaction().add(
  //     SystemProgram.transfer({
  //       fromPubkey: payer.publicKey,
  //       toPubkey: creatorToken0Account.address,
  //       lamports: initAmount0.toNumber(),
  //     }),
  //     createSyncNativeInstruction(creatorToken0Account.address)
  //   );

  //   await program.methods
  //     .initializeRaydiumLp(initAmount0, initAmount1, openTime)
  //     .accounts({
  //       cpSwapProgram: CP_SWAP_PROGRAM,
  //       creator: payer.publicKey,
  //       ammConfig: CONFIG_ADDRESS,
  //       authority: CP_SWAP_AUTHORITY,
  //       poolState: POOL_STATE,
  //       token0Mint: NATIVE_MINT,
  //       token1Mint: ASSET_B.publicKey, // MockToken mint
  //       lpMint: LP_MINT,
  //       creatorToken0: creatorToken0Account.address, // MockSOL account
  //       creatorToken1: creatorToken1Account.address, // MockToken account
  //       creatorLpToken: CREATOR_LP_TOKEN_ADDRESS,
  //       token0Vault: VAULT_0,
  //       token1Vault: VAULT_1,
  //       createPoolFee: CREATE_POOL_FEE_RECEIVE,
  //       observationState: OBSERVATION_STATE,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       token0Program: TOKEN_PROGRAM_ID,
  //       token1Program: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //       rent: SYSVAR_RENT_PUBKEY,
  //     })
  //     .preInstructions([
  //       ComputeBudgetProgram.setComputeUnitLimit({ units: 800000 }),
  //       ...tx.instructions,
  //     ])
  //     .rpc({
  //       skipPreflight: true,
  //     })
  //     .then((tx) => {
  //       console.log(tx);
  //     })
  //     .catch((err) => {
  //       console.log(err);
  //       throw err;
  //     });

  //   // use CP_SWAP_PROGRAM to get the pool state
  //   const accountInfo = await connection.getAccountInfo(POOL_STATE);

  //   const poolState = CpmmPoolInfoLayout.decode(accountInfo.data);
  //   const cpSwapPoolState = {
  //     ammConfig: poolState.configId,
  //     token0Mint: poolState.mintA,
  //     token0Program: poolState.mintProgramA,
  //     token1Mint: poolState.mintB,
  //     token1Program: poolState.mintProgramB,
  //   };
  //   console.log(cpSwapPoolState);
  // });

  it("Adds an asset to the pool", async () => {
    const [POOL_PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), payer.publicKey.toBuffer(), Buffer.from(POOL_NAME)],
      program.programId
    );

    const [ASSET_PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("asset"), POOL_PDA.toBuffer(), ASSET_A.publicKey.toBuffer()],
      program.programId
    );

    const [ASSET_VAULT_PDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("asset_vault"),
        POOL_PDA.toBuffer(),
        ASSET_A.publicKey.toBuffer(),
      ],
      program.programId
    );

    const depositAmount = new BN(1000).mul(new BN(10 ** 6));

    await program.methods
      .addAsset(depositAmount)
      .accounts({
        moonpool: MOONPOOL_PDA,
        pool: POOL_PDA,
        asset: ASSET_PDA,
        assetVault: ASSET_VAULT_PDA,
        payerTokenAccount: ASSET_TOKEN_ACCOUNT_A,
        payer: payer.publicKey,
        mint: ASSET_A.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
      .then((tx) => {
        console.log(tx);
      })
      .catch((err) => {
        console.log(err);
        throw err;
      });
  });

  it("Buys droplets", async () => {
    const [POOL_PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), payer.publicKey.toBuffer(), Buffer.from(POOL_NAME)],
      program.programId
    );

    const [POOL_VAULT] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool_vault"),
        payer.publicKey.toBuffer(),
        POOL_PDA.toBuffer(),
      ],
      program.programId
    );

    const [DROPLET_MINT] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("droplet_mint"),
        POOL_PDA.toBuffer(),
        Buffer.from(POOL_NAME),
      ],
      program.programId
    );

    const payerDropletTokenAccount = await getAssociatedTokenAddress(
      DROPLET_MINT,
      payer.publicKey,
      false
    );

    const feeVaultSolBalanceOne = await connection.getBalance(FEE_VAULT_PDA);
    console.log(feeVaultSolBalanceOne);

    // get pool owner from pool account
    const pool = await program.account.pool.fetch(POOL_PDA);
    const poolOwner = pool.owner;
    console.log(poolOwner.toBase58());

    await program.methods
      .buyDroplets(new BN(10000))
      .accounts({
        moonpool: MOONPOOL_PDA,
        feeVault: FEE_VAULT_PDA,
        pool: POOL_PDA,
        poolVault: POOL_VAULT,
        dropletMint: DROPLET_MINT,
        poolOwner: poolOwner,
        payerDropletTokenAccount: payerDropletTokenAccount,
        payer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
      .then((tx) => {
        console.log(tx);
      })
      .catch((err) => {
        console.log(err);
        throw err;
      });

    // sleep 30 seconds
    await new Promise((resolve) => setTimeout(resolve, 30000));

    const payerDropletBalance = await connection.getTokenAccountBalance(
      payerDropletTokenAccount
    );
    console.log(payerDropletBalance);

    const feeVaultSolBalance = await connection.getBalance(FEE_VAULT_PDA);
    console.log(feeVaultSolBalance);
  });

  it("Sells droplets", async () => {
    const [POOL_PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), payer.publicKey.toBuffer(), Buffer.from(POOL_NAME)],
      program.programId
    );

    const [POOL_VAULT] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool_vault"),
        payer.publicKey.toBuffer(),
        POOL_PDA.toBuffer(),
      ],
      program.programId
    );

    const [DROPLET_MINT] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("droplet_mint"),
        POOL_PDA.toBuffer(),
        Buffer.from(POOL_NAME),
      ],
      program.programId
    );

    const payerDropletTokenAccount = await getAssociatedTokenAddress(
      DROPLET_MINT,
      payer.publicKey,
      false
    );

    // get pool owner from pool account
    const pool = await program.account.pool.fetch(POOL_PDA);
    const poolOwner = pool.owner;
    console.log(poolOwner.toBase58());

    await program.methods
      .sellDroplets(new BN(10000))
      .accounts({
        moonpool: MOONPOOL_PDA,
        feeVault: FEE_VAULT_PDA,
        pool: POOL_PDA,
        poolVault: POOL_VAULT,
        dropletMint: DROPLET_MINT,
        poolOwner: poolOwner,
        payerDropletTokenAccount: payerDropletTokenAccount,
        payer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
      .then((tx) => {
        console.log(tx);
      })
      .catch((err) => {
        console.log(err);
        throw err;
      });

    // sleep 30 seconds
    await new Promise((resolve) => setTimeout(resolve, 30000));

    const payerDropletBalance = await connection.getTokenAccountBalance(
      payerDropletTokenAccount
    );
    console.log(payerDropletBalance);

    const payerSolBalance = await connection.getBalance(payer.publicKey);
    console.log(payerSolBalance);

    // memcmp all assets with the same pool
    const assets = await program.account.asset.all([
      {
        memcmp: {
          offset: 8,
          bytes: POOL_PDA.toBase58(),
        },
      },
    ]);

    assets.forEach(async (asset) => {
      let assetKey = asset.publicKey;
      console.log(assetKey.toBase58());
    });
  });
});
