import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Boosterfun } from "../target/types/boosterfun";
import { expect } from "chai";
import { TestValues, createValues, mintingTokens } from "./utils";

describe("Deposit liquidity", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
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
  });

  it("Deposit equal amounts", async () => {
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

    const depositTokenAccountA = await connection.getTokenAccountBalance(
      values.holderAccountA
    );
    expect(depositTokenAccountA.value.amount).to.equal(
      values.defaultSupply.sub(values.depositAmountA).toString()
    );
  });
});
