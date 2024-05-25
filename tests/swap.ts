import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Boosterfun } from "../target/types/boosterfun";
import { TestValues, createValues, mintingTokens, expectRevert } from "./utils";
import { BN } from "bn.js";

describe("Swap", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Boosterfun as Program<Boosterfun>;

  let values: TestValues;

  beforeEach(async () => {
    values = createValues();

    await program.methods
      .createAmm(values.id, values.fee, values.maxPerWallet)
      .accounts({ amm: values.ammKey, admin: values.admin.publicKey })
      .rpc();

    await mintingTokens({
      program,
      creator: values.admin,
      mintAKeypair: values.mintAKeypair,
    });

    await program.methods
      .createPool()
      .accounts({
        amm: values.ammKey,
        pool: values.poolKey,
        poolAuthority: values.poolAuthority,
        mintA: values.mintAKeypair.publicKey,
        poolAccountA: values.poolAccountA,
      })
      .rpc();

    await program.methods
      .depositLiquidity(values.depositAmountA)
      .accounts({
        pool: values.poolKey,
        poolAuthority: values.poolAuthority,
        depositor: values.admin.publicKey,
        mintA: values.mintAKeypair.publicKey,
        poolAccountA: values.poolAccountA,
        depositorAccountA: values.holderAccountA,
      })
      .signers([values.admin])
      .rpc({ skipPreflight: true });
  });

  it("Swap between tokens", async () => {
    console.log(`   ------ Buy token A with SOL ------`);

    const input = new BN(1).mul(new BN(10 ** 9));
    const tx = await program.methods
      .swapExactTokensForTokens(
        false,
        input,
        new BN(30000000).mul(new BN(10 ** 9))
      )
      .accounts({
        amm: values.ammKey,
        pool: values.poolKey,
        poolAuthority: values.poolAuthority,
        trader: values.admin.publicKey,
        mintA: values.mintAKeypair.publicKey,
        poolAccountA: values.poolAccountA,
        traderAccountA: values.holderAccountA,
        treasury: values.treasury.publicKey,
        treasuryAccountA: values.treasuryAccountA,
      })
      .signers([values.admin])
      .rpc({ skipPreflight: true });

    console.log(
      `     Transaction Signature: https://solscan.io/tx/${tx}?cluster=devnet`
    );

    console.log(`   ------ Sell token A to SOL ------- `);
    const sell_input = new BN(30000000).mul(new BN(10 ** 9));
    const sell_tx = await program.methods
      .swapExactTokensForTokens(
        true,
        sell_input,
        new BN(0.8).mul(new BN(10 ** 9))
      )
      .accounts({
        amm: values.ammKey,
        pool: values.poolKey,
        poolAuthority: values.poolAuthority,
        trader: values.admin.publicKey,
        mintA: values.mintAKeypair.publicKey,
        poolAccountA: values.poolAccountA,
        traderAccountA: values.holderAccountA,
        treasury: values.treasury.publicKey,
        treasuryAccountA: values.treasuryAccountA,
      })
      .signers([values.admin])
      .rpc({ skipPreflight: true });

    console.log(
      `     Transaction Signature: https://solscan.io/tx/${sell_tx}?cluster=devnet`
    );
  });

  it("Invalid buy too many tokens!", async () => {
    const input = new BN(5).mul(new BN(10 ** 9));
    await expectRevert(
      program.methods
        .swapExactTokensForTokens(
          false,
          input,
          new BN(100000000).mul(new BN(10 ** 9))
        )
        .accounts({
          amm: values.ammKey,
          pool: values.poolKey,
          poolAuthority: values.poolAuthority,
          trader: values.admin.publicKey,
          mintA: values.mintAKeypair.publicKey,
          poolAccountA: values.poolAccountA,
          traderAccountA: values.holderAccountA,
          treasury: values.treasury.publicKey,
          treasuryAccountA: values.treasuryAccountA,
        })
        .signers([values.admin])
        .rpc({ skipPreflight: true })
    );
  });
});
