import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

import {
  createAssociatedTokenAccountIfNotExist,
  createMarket,
  getAssociatedPoolKeys,
  getMarket,
  sleep,
} from "./util";

import { BN } from "bn.js";

import { Boosterfun } from "../target/types/boosterfun";
import {
  getAssociatedTokenAddress,
  getAccount,
  createBurnInstruction,
} from "@solana/spl-token";
import { mintingTokens } from "../tests/utils";
import { burn } from "@solana/spl-token";

const globalInfo = {
  marketProgram: new PublicKey("EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj"),
  ammProgram: new PublicKey("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8"),
  ammCreateFeeDestination: new PublicKey(
    "3XMrhbv989VxAMi3DErLV9eJht1pHppW5LbKxe9fkEFR"
  ),
  market: new Keypair(),
};

const confirmOptions = {
  skipPreflight: true,
};

describe("deploy to raydium", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const owner = anchor.Wallet.local().payer;
  const program = anchor.workspace.Boosterfun as Program<Boosterfun>;
  const marketId = globalInfo.market.publicKey.toString();
  const mintAKeypair = Keypair.generate();

  it("amm anchor test!", async () => {
    let conn = anchor.getProvider().connection;

    await mintingTokens({
      program,
      creator: owner,
      mintAKeypair,
    });
    // create serum market
    const marketInfo = await createMarket({
      connection: conn,
      wallet: anchor.Wallet.local(),
      baseMint: mintAKeypair.publicKey,
      quoteMint: new PublicKey("So11111111111111111111111111111111111111112"),
      baseLotSize: 1,
      quoteLotSize: 1,
      dexProgram: globalInfo.marketProgram,
      market: globalInfo.market,
    });
    // wait for transaction success
    sleep(60000);

    // get serum market info
    const market = await getMarket(
      conn,
      marketId,
      globalInfo.marketProgram.toString()
    );
    // console.log("market info:", JSON.stringify(market));

    const poolKeys = await getAssociatedPoolKeys({
      programId: globalInfo.ammProgram,
      serumProgramId: globalInfo.marketProgram,
      marketId: market.address,
      baseMint: market.baseMint,
      quoteMint: market.quoteMint,
    });
    // console.log("amm poolKeys: ", JSON.stringify(poolKeys));

    const ammAuthority = poolKeys.authority;
    const nonce = poolKeys.nonce;
    const ammId: PublicKey = poolKeys.id;
    const ammCoinVault: PublicKey = poolKeys.baseVault;
    const ammPcVault: PublicKey = poolKeys.quoteVault;
    const lpMintAddress: PublicKey = poolKeys.lpMint;
    const ammTargetOrders: PublicKey = poolKeys.targetOrders;
    const ammOpenOrders: PublicKey = poolKeys.openOrders;

    const [amm_config, _] = await getAmmConfigAddress(globalInfo.ammProgram);
    console.log("amm config:", amm_config.toString());
    /************************************ initialize test ***********************************************************************/

    const transaction = new Transaction();
    const userCoinTokenAccount = await createAssociatedTokenAccountIfNotExist(
      owner.publicKey,
      market.baseMint,
      transaction,
      anchor.getProvider().connection
    );

    const userPcTokenAccount = await createAssociatedTokenAccountIfNotExist(
      owner.publicKey,
      market.quoteMint,
      transaction,
      anchor.getProvider().connection
    );

    const userLPTokenAccount: PublicKey = await getAssociatedTokenAddress(
      poolKeys.lpMint,
      owner.publicKey
    );

    let tx = await program.methods
      .proxyInitialize(
        nonce,
        new anchor.BN(0),
        new BN(79).mul(new BN(10 ** 7)), // set as you want - wsol 0.79 wsol for test.
        new BN(20600000).mul(new BN(10 ** 9)) // set as you want - minted token
      )
      .accounts({
        ammProgram: globalInfo.ammProgram,
        amm: ammId,
        ammAuthority: ammAuthority,
        ammOpenOrders: ammOpenOrders,
        ammLpMint: lpMintAddress,
        ammCoinMint: market.baseMintAddress,
        ammPcMint: market.quoteMintAddress,
        ammCoinVault: ammCoinVault,
        ammPcVault: ammPcVault,
        ammTargetOrders: ammTargetOrders,
        ammConfig: amm_config,
        createFeeDestination: globalInfo.ammCreateFeeDestination,
        marketProgram: globalInfo.marketProgram,
        market: marketId,
        userWallet: owner.publicKey,
        userTokenCoin: userCoinTokenAccount,
        userTokenPc: userPcTokenAccount,
        userTokenLp: userLPTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        sysvarRent: SYSVAR_RENT_PUBKEY,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
      ])
      .rpc(confirmOptions);

    console.log(
      `     Transaction Signature: https://solscan.io/tx/${tx}?cluster=devnet`
    );

    const userLPTokenAccountInfo = await getAccount(conn, userLPTokenAccount);

    const total_lp = userLPTokenAccountInfo.amount;

    const burnInstruction = createBurnInstruction(
      userLPTokenAccount,
      poolKeys.lpMint,
      owner.publicKey,
      total_lp,
      [],
      TOKEN_PROGRAM_ID
    );

    const burn_transaction = new Transaction().add(burnInstruction);

    const burn_tx = await anchor.AnchorProvider.env().sendAndConfirm(
      burn_transaction,
      [owner]
    );

    console.log(
      `     Burn Transaction Signature: https://solscan.io/tx/${burn_tx}?cluster=devnet`
    );
  });
});

export async function getAmmConfigAddress(
  programId: PublicKey
): Promise<[PublicKey, number]> {
  const [address, bump] = await PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode("amm_config_account_seed"))],
    programId
  );
  return [address, bump];
}
