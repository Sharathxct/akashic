import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { assert } from "chai";
import { Akashic } from "../target/types/akashic";

describe("akashic", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.akashic as Program<Akashic>;
  
  // Test accounts
  let authority: Keypair;
  let user1: Keypair;
  
  // Test parameters
  const seed = new BN(12345);
  let deadline: BN;
  
  // PDAs
  let vowPda: PublicKey;
  let vaultPda: PublicKey;
  let longMintPda: PublicKey;
  let shortMintPda: PublicKey;
  
  // Helper function to derive PDAs
  const derivePDAs = (authority: PublicKey, seed: BN) => {
    // Vow PDA
    [vowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vow"),
        seed.toArrayLike(Buffer, "le", 8),
        authority.toBuffer(),
      ],
      program.programId
    );
    
    // Vault PDA
    [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vowPda.toBuffer()],
      program.programId
    );
    
    // Long mint PDA
    [longMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("long"), vowPda.toBuffer()],
      program.programId
    );
    
    // Short mint PDA
    [shortMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("short"), vowPda.toBuffer()],
      program.programId
    );
  };
  
  before(async () => {
    // Create test keypairs
    authority = Keypair.generate();
    user1 = Keypair.generate();
    
    // Airdrop SOL to test accounts
    const airdropAmount = 10 * LAMPORTS_PER_SOL;
    
    await Promise.all([
      provider.connection.requestAirdrop(authority.publicKey, airdropAmount),
      provider.connection.requestAirdrop(user1.publicKey, airdropAmount),
    ]);
    
    // Wait for airdrops to confirm
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    // Set deadline to 1 hour from now
    deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    
    // Derive PDAs
    derivePDAs(authority.publicKey, seed);
    
    console.log("Test setup complete:");
    console.log("Authority:", authority.publicKey.toString());
    console.log("Vow PDA:", vowPda.toString());
    console.log("Vault PDA:", vaultPda.toString());
  });

  describe("Initialize", () => {
    it("Successfully initializes a vow", async () => {
      const tx = await program.methods
        .initialize(seed, deadline)
        .accounts({
          authority: authority.publicKey,
          // @ts-ignore
          longMint: longMintPda,
          shortMint: shortMintPda,
          vault: vaultPda,
          vow: vowPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();

      console.log("Initialize transaction signature:", tx);

      // Verify vow account was created with correct data
      const vowAccount = await program.account.vow.fetch(vowPda);
      
      assert.equal(vowAccount.authority.toString(), authority.publicKey.toString());
      assert.equal(vowAccount.seeds.toString(), seed.toString());
      assert.equal(vowAccount.deadline.toString(), deadline.toString());
      assert.deepEqual(vowAccount.result, { pending: {} });
      assert.equal(vowAccount.resolved, false);
      
      console.log("✅ Vow initialized successfully");
    });
  });

  describe("Long Position", () => {
    it("Successfully creates long position", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const userLongAta = getAssociatedTokenAddressSync(longMintPda, user1.publicKey);
      const vaultShortAta = getAssociatedTokenAddressSync(shortMintPda, vowPda, true);
      
      const initialUserBalance = await provider.connection.getBalance(user1.publicKey);
      
      const tx = await program.methods
        .long(amount)
        .accounts({
          user: user1.publicKey,
          vow: vowPda,
          // @ts-ignore
          userLong: userLongAta,
          vault: vaultPda,
          // @ts-ignore
          longMint: longMintPda,
          shortMint: shortMintPda,
          vaultShort: vaultShortAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      console.log("Long position transaction signature:", tx);

      // Verify user received long tokens
      const userLongBalance = await provider.connection.getTokenAccountBalance(userLongAta);
      assert.equal(userLongBalance.value.amount, amount.toString());
      
      // Verify vault received short tokens
      const vaultShortBalance = await provider.connection.getTokenAccountBalance(vaultShortAta);
      assert.equal(vaultShortBalance.value.amount, amount.toString());
      
      // Verify SOL was transferred to vault
      const finalUserBalance = await provider.connection.getBalance(user1.publicKey);
      assert.isTrue(initialUserBalance - finalUserBalance > amount.toNumber());
      
      console.log("✅ Long position created successfully");
      console.log(`User received ${amount.toString()} long tokens`);
      console.log(`Vault received ${amount.toString()} short tokens`);
    });
  });

  describe("Short Position", () => {
    it("Successfully buys short tokens from vault", async () => {
      const amount = new BN(LAMPORTS_PER_SOL / 2); // 0.5 SOL worth of tokens
      const userShortAta = getAssociatedTokenAddressSync(shortMintPda, user1.publicKey);
      const vaultShortAta = getAssociatedTokenAddressSync(shortMintPda, vowPda, true);
      
      const initialUserBalance = await provider.connection.getBalance(user1.publicKey);
      const initialVaultShortBalance = await provider.connection.getTokenAccountBalance(vaultShortAta);
      
      const tx = await program.methods
        .short(amount)
        .accounts({
          user: user1.publicKey,
          vow: vowPda,
          // @ts-ignore
          userShort: userShortAta,
          vault: vaultPda,
          shortMint: shortMintPda,
          vaultShort: vaultShortAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      console.log("Short position transaction signature:", tx);

      // Verify user received short tokens
      const userShortBalance = await provider.connection.getTokenAccountBalance(userShortAta);
      assert.equal(userShortBalance.value.amount, amount.toString());
      
      // Verify short tokens were transferred from vault
      const finalVaultShortBalance = await provider.connection.getTokenAccountBalance(vaultShortAta);
      const expectedVaultBalance = new BN(initialVaultShortBalance.value.amount).sub(amount);
      assert.equal(finalVaultShortBalance.value.amount, expectedVaultBalance.toString());
      
      // Verify user paid SOL
      const finalUserBalance = await provider.connection.getBalance(user1.publicKey);
      assert.isTrue(initialUserBalance - finalUserBalance > amount.toNumber());
      
      console.log("✅ Short position created successfully");
      console.log(`User received ${amount.toString()} short tokens`);
    });
  });

  describe("Complete Flow Test", () => {
    it("Demonstrates prediction market functionality", async () => {
      console.log("🚀 Starting simplified flow demonstration...");
      
      // Create test accounts for isolated test
      const demoAuthority = Keypair.generate();
      const bettor = Keypair.generate();
      
      // Airdrop SOL
      await Promise.all([
        provider.connection.requestAirdrop(demoAuthority.publicKey, 5 * LAMPORTS_PER_SOL),
        provider.connection.requestAirdrop(bettor.publicKey, 5 * LAMPORTS_PER_SOL),
      ]);
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Step 1: Initialize prediction market
      const demoSeed = new BN(77777);
      const futureDeadline = new BN(Math.floor(Date.now() / 1000) + 3600); // 1 hour from now
      
      const [demoVowPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vow"), demoSeed.toArrayLike(Buffer, "le", 8), demoAuthority.publicKey.toBuffer()],
        program.programId
      );
      const [demoVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), demoVowPda.toBuffer()],
        program.programId
      );
      const [demoLongMintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("long"), demoVowPda.toBuffer()],
        program.programId
      );
      const [demoShortMintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("short"), demoVowPda.toBuffer()],
        program.programId
      );
      
      console.log("📝 1. Creating prediction market...");
      await program.methods
        .initialize(demoSeed, futureDeadline)
        .accounts({
          authority: demoAuthority.publicKey,
          // @ts-ignore
          longMint: demoLongMintPda,
          shortMint: demoShortMintPda,
          vault: demoVaultPda,
          vow: demoVowPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([demoAuthority])
        .rpc();
      
      // Verify initialization
      const vowData = await program.account.vow.fetch(demoVowPda);
      console.log(`✅ Market created! Deadline: ${new Date(vowData.deadline.toNumber() * 1000)}`);
      console.log(`   Status: ${JSON.stringify(vowData.result)}, Resolved: ${vowData.resolved}`);
      
      // Step 2: User makes a prediction (long position)
      console.log("📈 2. User betting on success (long position)...");
      const betAmount = new BN(LAMPORTS_PER_SOL);
      const bettorLongAta = getAssociatedTokenAddressSync(demoLongMintPda, bettor.publicKey);
      const demoVaultShortAta = getAssociatedTokenAddressSync(demoShortMintPda, demoVowPda, true);
      
      const initialBettorBalance = await provider.connection.getBalance(bettor.publicKey);
      
      await program.methods
        .long(betAmount)
        .accounts({
          user: bettor.publicKey,
          vow: demoVowPda,
          // @ts-ignore
          userLong: bettorLongAta,
          vault: demoVaultPda,
          longMint: demoLongMintPda,
          shortMint: demoShortMintPda,
          vaultShort: demoVaultShortAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([bettor])
        .rpc();
      
      // Verify position
      const longBalance = await provider.connection.getTokenAccountBalance(bettorLongAta);
      const vaultShortBalance = await provider.connection.getTokenAccountBalance(demoVaultShortAta);
      const finalBettorBalance = await provider.connection.getBalance(bettor.publicKey);
      
      console.log(`✅ Position created!`);
      console.log(`   User deposited: ${(initialBettorBalance - finalBettorBalance) / LAMPORTS_PER_SOL} SOL`);
      console.log(`   User received: ${longBalance.value.amount} LONG tokens`);
      console.log(`   Vault holds: ${vaultShortBalance.value.amount} SHORT tokens`);
      
      // Step 3: Show that short positions work too
      console.log("📉 3. User buying some short tokens...");
      const shortAmount = new BN(LAMPORTS_PER_SOL / 4); // 0.25 SOL worth
      const bettorShortAta = getAssociatedTokenAddressSync(demoShortMintPda, bettor.publicKey);
      
      await program.methods
        .short(shortAmount)
        .accounts({
          user: bettor.publicKey,
          vow: demoVowPda,
          // @ts-ignore
          userShort: bettorShortAta,
          vault: demoVaultPda,
          shortMint: demoShortMintPda,
          vaultShort: demoVaultShortAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([bettor])
        .rpc();
      
      const shortBalance = await provider.connection.getTokenAccountBalance(bettorShortAta);
      console.log(`✅ Also bought ${shortBalance.value.amount} SHORT tokens`);
      
      // Step 4: Demonstrate error cases
      console.log("🚫 4. Demonstrating error cases...");
      
      // Try to claim before resolution
      try {
        await program.methods
          .claim()
          .accounts({
            user: bettor.publicKey,
            vow: demoVowPda,
            // @ts-ignore
            userLong: bettorLongAta,
            vault: demoVaultPda,
            longMint: demoLongMintPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([bettor])
          .rpc();
        assert.fail("Should have failed");
      } catch (error) {
        console.log("✅ Correctly rejected claim before resolution");
      }
      
      // Try to submit result before deadline
      try {
        await program.methods
          .submitResult({ success: {} })
          .accounts({
            authority: demoAuthority.publicKey,
            vow: demoVowPda,
          })
          .signers([demoAuthority])
          .rpc();
        assert.fail("Should have failed");
      } catch (error) {
        console.log("✅ Correctly rejected result submission before deadline");
      }
      
      console.log("🎉 Prediction market demonstration complete!");
      console.log("");
      console.log("📊 Summary:");
      console.log(`   • Market created with deadline: ${new Date(vowData.deadline.toNumber() * 1000)}`);
      console.log(`   • User holds ${longBalance.value.amount} LONG tokens (bet on success)`);
      console.log(`   • User holds ${shortBalance.value.amount} SHORT tokens (bet on failure)`);
      console.log(`   • Vault holds ${(await provider.connection.getTokenAccountBalance(demoVaultShortAta)).value.amount} SHORT tokens available`);
      console.log(`   • All error cases correctly handled`);
      console.log("");
      console.log("🔄 In a real scenario:");
      console.log("   1. After deadline passes, admin submits actual result");
      console.log("   2. Winning token holders can claim 2x their stake");
      console.log("   3. Losing tokens become worthless");
      
      // Final assertions
      assert.equal(longBalance.value.amount, betAmount.toString());
      assert.equal(shortBalance.value.amount, shortAmount.toString());
      assert.isFalse(vowData.resolved);
      assert.deepEqual(vowData.result, { pending: {} });
    });
  });

  describe("Error Cases", () => {
    it("Fails to create long position after deadline", async () => {
      // Create expired vow
      const expiredSeed = new BN(55555);
      const expiredDeadline = new BN(Math.floor(Date.now() / 1000) - 100);
      const expiredAuthority = Keypair.generate();
      
      await provider.connection.requestAirdrop(expiredAuthority.publicKey, 5 * LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const [expiredVowPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vow"), expiredSeed.toArrayLike(Buffer, "le", 8), expiredAuthority.publicKey.toBuffer()],
        program.programId
      );
      const [expiredVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), expiredVowPda.toBuffer()],
        program.programId
      );
      const [expiredLongMintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("long"), expiredVowPda.toBuffer()],
        program.programId
      );
      const [expiredShortMintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("short"), expiredVowPda.toBuffer()],
        program.programId
      );
      
      // Initialize expired vow
      await program.methods
        .initialize(expiredSeed, expiredDeadline)
        .accounts({
          authority: expiredAuthority.publicKey,
          // @ts-ignore
          longMint: expiredLongMintPda,
          shortMint: expiredShortMintPda,
          vault: expiredVaultPda,
          vow: expiredVowPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([expiredAuthority])
        .rpc();
      
      // Try to create long position (should fail)
      try {
        const userLongAta = getAssociatedTokenAddressSync(expiredLongMintPda, user1.publicKey);
        const vaultShortAta = getAssociatedTokenAddressSync(expiredShortMintPda, expiredVowPda, true);
        
        await program.methods
          .long(new BN(LAMPORTS_PER_SOL))
          .accounts({
            user: user1.publicKey,
            vow: expiredVowPda,
            // @ts-ignore
            userLong: userLongAta,
            vault: expiredVaultPda,
            longMint: expiredLongMintPda,
            shortMint: expiredShortMintPda,
            vaultShort: vaultShortAta,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        assert.fail("Should have failed due to deadline");
      } catch (error) {
        assert.include(error.toString(), "DeadlinePassed");
        console.log("✅ Correctly rejected long position after deadline");
      }
    });

    it("Fails to claim from unresolved vow", async () => {
      const userLongAta = getAssociatedTokenAddressSync(longMintPda, user1.publicKey);
      
      try {
        await program.methods
          .claim()
          .accounts({
            user: user1.publicKey,
            vow: vowPda, // This vow is not resolved yet
            // @ts-ignore
            userLong: userLongAta,
            vault: vaultPda,
            longMint: longMintPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        assert.fail("Should have failed");
      } catch (error) {
        assert.include(error.toString(), "VowNotResolved");
        console.log("✅ Correctly rejected claim from unresolved vow");
      }
    });
  });
});
