import { expect, describe, beforeAll, test } from "@jest/globals";
import * as anchor from "@coral-xyz/anchor";
import { type Program, BN } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  type TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getAssociatedTokenAddress,
  getMinimumBalanceForRentExemptMint,
  getAccount,
} from "@solana/spl-token";

import { randomBytes } from "crypto";

import { confirmTransaction, makeKeypairs } from "@solana-developers/helpers";

const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
  TOKEN_2022_PROGRAM_ID;

export const getRandomBigNumber = (size: number = 8) => {
  return new BN(randomBytes(size));
};

function areBnEqual(a: unknown, b: unknown): boolean | undefined {
  const isABn = a instanceof BN;
  const isBBn = b instanceof BN;

  if (isABn && isBBn) {
    return a.eq(b);
  } else if (isABn === isBBn) {
    return undefined;
  } else {
    return false;
  }
}
expect.addEqualityTesters([areBnEqual]);

const createTokenAndMintTo = async (
  connection: Connection,
  payer: PublicKey,
  tokenMint: PublicKey,
  decimals: number,
  mintAuthority: PublicKey,
  mintTo: Array<{ recepient: PublicKey; amount: number }>
): Promise<Array<TransactionInstruction>> => {
  let minimumLamports = await getMinimumBalanceForRentExemptMint(connection);

  let createTokeIxs = [
    SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: tokenMint,
      lamports: minimumLamports,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM,
    }),
    createInitializeMint2Instruction(
      tokenMint,
      decimals,
      mintAuthority,
      null,
      TOKEN_PROGRAM
    ),
  ];

  let mintToIxs = mintTo.flatMap(({ recepient, amount }) => {
    const ataAddress = getAssociatedTokenAddressSync(
      tokenMint,
      recepient,
      false,
      TOKEN_PROGRAM
    );

    return [
      createAssociatedTokenAccountIdempotentInstruction(
        payer,
        ataAddress,
        recepient,
        tokenMint,
        TOKEN_PROGRAM
      ),
      createMintToInstruction(
        tokenMint,
        ataAddress,
        mintAuthority,
        amount,
        [],
        TOKEN_PROGRAM
      ),
    ];
  });

  return [...createTokeIxs, ...mintToIxs];
};

const getTokenBalanceOn = (
  connection: Connection,
) => async (
  tokenAccountAddress: PublicKey,
): Promise<BN> => {
  const tokenBalance = await connection.getTokenAccountBalance(tokenAccountAddress);
  return new BN(tokenBalance.value.amount);
};

// Jest debug console it too verbose.
// const jestConsole = console;

describe("escrow", () => {
  // Use the cluster and the keypair from Anchor.toml
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();

  // See https://github.com/coral-xyz/anchor/issues/3122
  // const user = (provider.wallet as anchor.Wallet).payer;
  // const payer = user;

  const connection = provider.connection;

  const program = anchor.workspace.Escrow as Program<Escrow>;

  const [alice, bob, usdcMint, wifMint] = makeKeypairs(4);

  const [aliceUsdcAccount, aliceWifAccount, bobUsdcAccount, bobWifAccount] = [
    alice,
    bob,
  ].flatMap((owner) =>
    [usdcMint, wifMint].map((tokenMint) =>
      getAssociatedTokenAddressSync(
        tokenMint.publicKey,
        owner.publicKey,
        false,
        TOKEN_PROGRAM
      )
    )
  );

  // Creates Alice and Bob accounts, 2 token mints, and associated token
  // accounts for both tokens for both users.
  beforeAll(async () => {
    // global.console = require('console');

    const giveAliceAndBobSolIxs: Array<TransactionInstruction> = [
      alice,
      bob,
    ].map((owner) =>
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: owner.publicKey,
        lamports: 10 * LAMPORTS_PER_SOL,
      })
    );

    const usdcSetupIxs = await createTokenAndMintTo(
      connection,
      provider.publicKey,
      usdcMint.publicKey,
      6,
      alice.publicKey,
      [
        { recepient: alice.publicKey, amount: 100_000_000_000 },
        { recepient: bob.publicKey, amount: 20_000_000_000 },
      ]
    );

    const wifSetupIxs = await createTokenAndMintTo(
      connection,
      provider.publicKey,
      wifMint.publicKey,
      6,
      bob.publicKey,
      [
        { recepient: alice.publicKey, amount: 5_000_000_000 },
        { recepient: bob.publicKey, amount: 300_000_000_000 },
      ]
    );

    // Add all these instructions to our transaction
    let tx = new Transaction();
    tx.instructions = [
      ...giveAliceAndBobSolIxs,
      ...usdcSetupIxs,
      ...wifSetupIxs,
    ];

    const _setupTxSig = await provider.sendAndConfirm(tx, [
      alice,
      bob,
      usdcMint,
      wifMint,
    ]);
  });

  const makeOfferTx = async (
    maker: Keypair,
    offerId: BN,
    offeredTokenMint: PublicKey,
    offeredAmount: BN,
    wantedTokenMint: PublicKey,
    wantedAmount: BN
  ): Promise<PublicKey> => {
    const transactionSignature = await program.methods
      .makeOffer(offerId, offeredAmount, wantedAmount)
      .accounts({
        maker: maker.publicKey,
        tokenMintA: offeredTokenMint,
        tokenMintB: wantedTokenMint,
        // As the `token_program` account is specified as
        //
        //   pub token_program: Interface<'info, TokenInterface>,
        //
        // the client library needs us to provide the specific program address
        // explicitly.
        //
        // This is unlike the `associated_token_program` or the `system_program`
        // account addresses, that are specified in the program IDL, as they are
        // expected to reference the same programs for all the `makeOffer`
        // invocations.
        tokenProgram: TOKEN_PROGRAM,
      })
      .signers([maker])
      .rpc();

    await confirmTransaction(connection, transactionSignature);

    const [offerAddress, _offerBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"),
        maker.publicKey.toBuffer(),
        offerId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    return offerAddress;
  };

  const takeOfferTx = async (
    offerAddress: PublicKey,
    taker: Keypair,
  ): Promise<void> => {

    // `accounts` argument debugging tool.  Should be part of Anchor really.
    //
    // type FlatType<T> = T extends object
    //   ? { [K in keyof T]: FlatType<T[K]> }
    //   : T;
    //
    // type AccountsArgs = FlatType<
    //   Parameters<
    //     ReturnType<
    //       Program<Escrow>["methods"]["takeOffer"]
    //     >["accounts"]
    //   >
    // >;
    const transactionSignature = await program.methods
      .takeOffer()
      .accounts({
        taker: taker.publicKey,
        offer: offerAddress,
        // See note in the `makeOfferTx` on why this program address is provided
        // and the rest are not.
        tokenProgram: TOKEN_PROGRAM,
      })
      .signers([taker])
      .rpc();

    await confirmTransaction(connection, transactionSignature);
  };

  const cancelOfferTx = async (
    offerAddress: PublicKey,
    maker: Keypair,
  ): Promise<void> => {
    const transactionSignature = await program.methods
      .cancelOffer()
      .accounts({
        offer: offerAddress,
        tokenProgram: TOKEN_PROGRAM,
      })
      .accountsPartial({
        maker: maker.publicKey,
      })
      .signers([maker])
      .rpc();

    await confirmTransaction(connection, transactionSignature);
  };

  const getTokenBalance = getTokenBalanceOn(connection);

  const getTokenAccount = async (
    address: PublicKey,
  ) => {
    return await getAccount(connection, address);
  };

  let offeredUsdc;
  let wantedWif;
  let offerAddress;
  let offerId;

  let aliceUsdcAccountBeforeOffer;
  let aliceWifAccountBeforeOffer;
  let bobUsdcAccountBeforeOffer;
  let bobWifAccountBeforeOffer;

  beforeEach(async() => {
    offeredUsdc = new BN(10_000_000);
    wantedWif = new BN(100_000_000);

    aliceUsdcAccountBeforeOffer = await getTokenBalance(aliceUsdcAccount);
    aliceWifAccountBeforeOffer = await getTokenBalance(aliceWifAccount);
    bobUsdcAccountBeforeOffer = await getTokenBalance(bobUsdcAccount);
    bobWifAccountBeforeOffer = await getTokenBalance(bobWifAccount);

    // Pick a random ID for the new offer.
    offerId = getRandomBigNumber();

    offerAddress = await makeOfferTx(
      alice,
      offerId,
      usdcMint.publicKey,
      offeredUsdc,
      wifMint.publicKey,
      wantedWif
    );
  });

  test("Offer created by Alice, Tokens approved for the offer", async () => {
    expect(await getTokenBalance(aliceUsdcAccount)).toEqual(aliceUsdcAccountBeforeOffer);
    expect(await getTokenBalance(aliceWifAccount)).toEqual(aliceWifAccountBeforeOffer);
    expect(await getTokenBalance(bobUsdcAccount)).toEqual(bobUsdcAccountBeforeOffer);
    expect(await getTokenBalance(bobWifAccount)).toEqual(bobWifAccountBeforeOffer);

    const offerAccount = await program.account.offer.fetch(offerAddress);
    expect(offerAccount.maker).toEqual(alice.publicKey);
    expect(offerAccount.tokenMintA).toEqual(usdcMint.publicKey);
    expect(offerAccount.tokenMintB).toEqual(wifMint.publicKey);
    expect(offerAccount.tokenBWantedAmount).toEqual(wantedWif);
  });

  test("Offer taken by Bob, tokens balances are updated", async () => {
    expect(await getTokenBalance(aliceUsdcAccount)).toEqual(aliceUsdcAccountBeforeOffer);
    expect(await getTokenBalance(aliceWifAccount)).toEqual(aliceWifAccountBeforeOffer);
    expect(await getTokenBalance(bobUsdcAccount)).toEqual(bobUsdcAccountBeforeOffer);
    expect(await getTokenBalance(bobWifAccount)).toEqual(bobWifAccountBeforeOffer);

    await takeOfferTx(offerAddress, bob);

    expect(await getTokenBalance(aliceUsdcAccount)).toEqual(aliceUsdcAccountBeforeOffer.sub(offeredUsdc));
    expect(await getTokenBalance(aliceWifAccount)).toEqual(aliceWifAccountBeforeOffer.add(wantedWif));
    expect(await getTokenBalance(bobUsdcAccount)).toEqual(bobUsdcAccountBeforeOffer.add(offeredUsdc));
    expect(await getTokenBalance(bobWifAccount)).toEqual(bobWifAccountBeforeOffer.sub(wantedWif));
  });

  test("Offer created by Alice and cancel offers", async () => {
    expect(await getTokenBalance(aliceUsdcAccount)).toEqual(aliceUsdcAccountBeforeOffer);
    expect(await getTokenBalance(aliceWifAccount)).toEqual(aliceWifAccountBeforeOffer);
    expect(await getTokenBalance(bobUsdcAccount)).toEqual(bobUsdcAccountBeforeOffer);
    expect(await getTokenBalance(bobWifAccount)).toEqual(bobWifAccountBeforeOffer);

    await cancelOfferTx(offerAddress, alice);

    expect(await getTokenBalance(aliceUsdcAccount)).toEqual(aliceUsdcAccountBeforeOffer);
    expect(await getTokenBalance(aliceWifAccount)).toEqual(aliceWifAccountBeforeOffer);
    expect(await getTokenBalance(bobUsdcAccount)).toEqual(bobUsdcAccountBeforeOffer);
    expect(await getTokenBalance(bobWifAccount)).toEqual(bobWifAccountBeforeOffer);
  });
 });
