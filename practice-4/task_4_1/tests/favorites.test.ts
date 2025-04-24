import * as anchor from "@coral-xyz/anchor";
import {Program, web3} from "@coral-xyz/anchor";
import {Favorites} from "../target/types/favorites";
import {airdropIfRequired, getCustomErrorMessage} from "@solana-developers/helpers";
import {expect, describe} from "@jest/globals";
import {systemProgramErrors} from "./system-program-errors";

describe("favorites", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const provider = anchor.getProvider();
    const connection = provider.connection;
    const program = anchor.workspace.Favorites as Program<Favorites>;

    let user;
    let favoriteNumber;
    let favoriteColor;

    beforeEach(async () => {
        user = web3.Keypair.generate();

        await airdropIfRequired(
            connection,
            user.publicKey,
            0.5 * web3.LAMPORTS_PER_SOL,
            web3.LAMPORTS_PER_SOL
        );

        favoriteNumber = new anchor.BN(23);
        favoriteColor = "red";
    }, 30000);

    it("Writes our favorites to the blockchain", async () => {
        let tx: string | null = null;
        try {
            tx = await program.methods
                .setFavorites(favoriteNumber, favoriteColor)
                .accounts({
                    user: user.publicKey,
                })
                .signers([user])
                .rpc();
        } catch (thrownObject) {
            const rawError = thrownObject as Error;
            throw new Error(getCustomErrorMessage(systemProgramErrors, rawError.message))
        }

        console.log(`Tx signature: ${tx}`);

        const [favoritesPda, _favoritesBump] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("favorites"), user.publicKey.toBuffer()],
            program.programId
        );

        const dataFromPda = await program.account.favorites.fetch((favoritesPda));
        expect(dataFromPda.color).toEqual(favoriteColor);
        expect(dataFromPda.number.toNumber()).toEqual(favoriteNumber.toNumber());
    });

    it("Update our favorites to the blockchain", async () => {
        let tx: string | null = null;
        try {
            tx = await program.methods
                .setFavorites(favoriteNumber, favoriteColor)
                .accounts({
                    user: user.publicKey,
                })
                .signers([user])
                .rpc();
        } catch (thrownObject) {
            const rawError = thrownObject as Error;
            throw new Error(getCustomErrorMessage(systemProgramErrors, rawError.message))
        }

        const [favoritesPda, _favoritesBump] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("favorites"), user.publicKey.toBuffer()],
            program.programId
        );

        const updateFavoriteNumber = new anchor.BN(25);
        const updateFavoriteColor = "green";

        try {
            tx = await program.methods
                .updateFavorites(updateFavoriteNumber, updateFavoriteColor)
                .accounts({
                    user: user.publicKey,
                    favorites: favoritesPda,
                })
                .signers([user])
                .rpc();
        } catch (thrownObject) {
            const rawError = thrownObject as Error;
            throw new Error(getCustomErrorMessage(systemProgramErrors, rawError.message))
        }

        console.log(`Tx signature: ${tx}`);

        const dataFromPda = await program.account.favorites.fetch((favoritesPda));
        expect(dataFromPda.color).toEqual(updateFavoriteColor);
        expect(dataFromPda.number.toNumber()).toEqual(updateFavoriteNumber.toNumber());
    });

    it("Update our favorite color to the blockchain", async () => {
        let tx: string | null = null;
        try {
            tx = await program.methods
                .setFavorites(favoriteNumber, favoriteColor)
                .accounts({
                    user: user.publicKey,
                })
                .signers([user])
                .rpc();
        } catch (thrownObject) {
            const rawError = thrownObject as Error;
            throw new Error(getCustomErrorMessage(systemProgramErrors, rawError.message))
        }

        const [favoritesPda, _favoritesBump] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("favorites"), user.publicKey.toBuffer()],
            program.programId
        );

        const updateFavoriteNumber = null;
        const updateFavoriteColor = "green";

        try {
            tx = await program.methods
                .updateFavorites(updateFavoriteNumber, updateFavoriteColor)
                .accounts({
                    user: user.publicKey,
                    favorites: favoritesPda
                })
                .signers([user])
                .rpc();
        } catch (thrownObject) {
            const rawError = thrownObject as Error;
            throw new Error(getCustomErrorMessage(systemProgramErrors, rawError.message))
        }

        console.log(`Tx signature: ${tx}`);

        const dataFromPda = await program.account.favorites.fetch((favoritesPda));
        expect(dataFromPda.color).toEqual(updateFavoriteColor);
        expect(dataFromPda.number.toNumber()).toEqual(favoriteNumber.toNumber());
    });

    it("Update our favorite number to the blockchain", async () => {
        let tx: string | null = null;
        try {
            tx = await program.methods
                .setFavorites(favoriteNumber, favoriteColor)
                .accounts({
                    user: user.publicKey,
                })
                .signers([user])
                .rpc();
        } catch (thrownObject) {
            const rawError = thrownObject as Error;
            throw new Error(getCustomErrorMessage(systemProgramErrors, rawError.message))
        }

        const [favoritesPda, _favoritesBump] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("favorites"), user.publicKey.toBuffer()],
            program.programId
        );

        const updateFavoriteNumber = new anchor.BN(28);
        const updateFavoriteColor = null;

        try {
            tx = await program.methods
                .updateFavorites(updateFavoriteNumber, updateFavoriteColor)
                .accounts({
                    user: user.publicKey,
                    favorites: favoritesPda,
                })
                .signers([user])
                .rpc();
        } catch (thrownObject) {
            const rawError = thrownObject as Error;
            throw new Error(getCustomErrorMessage(systemProgramErrors, rawError.message))
        }

        console.log(`Tx signature: ${tx}`);

        const dataFromPda = await program.account.favorites.fetch((favoritesPda));
        expect(dataFromPda.color).toEqual(favoriteColor);
        expect(dataFromPda.number.toNumber()).toEqual(updateFavoriteNumber.toNumber());
    });
});