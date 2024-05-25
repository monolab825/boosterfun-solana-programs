import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Boosterfun } from "../target/types/boosterfun";
import { TestValues, createValues, mintingTokens } from "./utils";

describe("Create and mint token with metadata", () => {
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
  });
});
