import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Boosterfun } from "../target/types/boosterfun";
import { TestValues, createValues, mintingTokens } from "./utils";

describe("Create pool", () => {
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
  });

  it("Creation", async () => {
    await program.methods
      .createPool()
      .accounts({
        amm: values.ammKey,
        pool: values.poolKey,
        poolAuthority: values.poolAuthority,
        mintA: values.mintAKeypair.publicKey,
        poolAccountA: values.poolAccountA,
      })
      .rpc({ skipPreflight: true });
  });
});
