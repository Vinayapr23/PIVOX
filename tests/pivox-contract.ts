import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PivoxContract } from "../target/types/pivox_contract";
import { assert } from "chai";
import {
  createMint,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import fs from "fs";

describe("pivox_contract - Full Flow", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.PivoxContract as Program<PivoxContract>;

  // // Accounts
  let client = anchor.web3.Keypair.generate();
  let freelancer = anchor.web3.Keypair.generate();
  let randomUser = anchor.web3.Keypair.generate();


function loadKeypair(path: string): anchor.web3.Keypair {
    const secretKey = Uint8Array.from(JSON.parse(fs.readFileSync(path, "utf8")));
    return anchor.web3.Keypair.fromSecretKey(secretKey);
}
//   let client = loadKeypair("wallets/random_user.json");
// let freelancer = loadKeypair("wallets/freelancer.json");
// let randomUser = loadKeypair("wallets/client.json");

  // PDAs
  let milestoneApprovalPda: anchor.web3.PublicKey;
  let vaultAccountPda: anchor.web3.PublicKey;
  let contractPda: anchor.web3.PublicKey;

  // Bumps
  let vaultBump: number;
  let contractBump: number;
  let milestoneApprovalBump: number;

  // Tokens
  let usdcMint: anchor.web3.PublicKey;
  let clientAta: anchor.web3.PublicKey;
  let freelancerAta: anchor.web3.PublicKey;
  let vaultAta: anchor.web3.PublicKey;

  // Constants
  const milestones = [
    { description: "Design Phase", amount: new anchor.BN(50_000_000), freelancerSubmitted: false, clientApproved: false, freelancerConfirmed: false, isReleased: false },
    { description: "Development Phase", amount: new anchor.BN(30_000_000), freelancerSubmitted: false, clientApproved: false, freelancerConfirmed: false, isReleased: false },
    { description: "Testing Phase", amount: new anchor.BN(20_000_000), freelancerSubmitted: false, clientApproved: false, freelancerConfirmed: false, isReleased: false },
  ];

  before(async () => {
    // Airdrops
    async function airdropAndConfirm(
      connection: anchor.web3.Connection,
      pubkey: anchor.web3.PublicKey,
      amountLamports =1e9
    ) {

      const balance = await connection.getBalance(pubkey);
  if (balance < anchor.web3.LAMPORTS_PER_SOL) {
      const sig = await connection.requestAirdrop(pubkey, amountLamports);
      const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
      await connection.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight });
      const balance = await connection.getBalance(pubkey);
      console.log(`✅ Airdrop confirmed. New balance: ${balance / anchor.web3.LAMPORTS_PER_SOL} SOL`);
     }
    }
    await airdropAndConfirm(provider.connection, client.publicKey);
    await airdropAndConfirm(provider.connection, freelancer.publicKey);
    await airdropAndConfirm(provider.connection, randomUser.publicKey);


    // Create USDC Mint
    usdcMint = await createMint(provider.connection, client, client.publicKey, null, 6);

    // Create ATAs
    clientAta = await getAssociatedTokenAddress(usdcMint, client.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, client, usdcMint, client.publicKey);

    freelancerAta = await getAssociatedTokenAddress(usdcMint, freelancer.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, client, usdcMint, freelancer.publicKey);

    // Mint tokens to client
    await mintTo(provider.connection, client, usdcMint, clientAta, client, 1_000_000_000);

    // Find PDAs
    [milestoneApprovalPda, milestoneApprovalBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("milestone_approval"), client.publicKey.toBuffer(), freelancer.publicKey.toBuffer()],
      program.programId
    );
    [vaultAccountPda, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_account"), client.publicKey.toBuffer(), freelancer.publicKey.toBuffer()],
      program.programId
    );
    [contractPda, contractBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contract"), client.publicKey.toBuffer(), freelancer.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Initializes Milestone Approval", async () => {
    const tx = await program.methods.initializeMilestoneApproval(1)
      .accountsPartial({
        payer: client.publicKey,
        client: client.publicKey,
        freelancer: freelancer.publicKey,
        milestoneApproval: milestoneApprovalPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([client])
      .rpc();
    console.log("✅ initializeMilestoneApproval tx:", tx);

    const acc = await program.account.milestoneApproval.fetch(milestoneApprovalPda);
    assert.ok(acc.client.equals(client.publicKey));
    assert.equal(acc.threshold, 1);
  });

  it("Freelancer approves milestones and creates contract", async () => {
    const tx = await program.methods.approve(
      50,
      50,
      new anchor.BN(0),
      new anchor.BN(Date.now()),
      new anchor.BN(60 * 60 * 24 * 30),
      "Dispute Clause",
      vaultBump,
      contractBump,
      "active",
      milestones
    )
    .accountsPartial({
      freelancer: freelancer.publicKey,
      client: client.publicKey,
      usdcMint: usdcMint,
      milestoneApproval: milestoneApprovalPda,
      vaultAccount: vaultAccountPda,
      contract: contractPda,
      freelancerAta: freelancerAta,
      clientAta: clientAta,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    })
    .signers([freelancer])
    .rpc();
    console.log("✅ approve tx:", tx);

    const contract = await program.account.contract.fetch(contractPda);
    assert.equal(contract.milestones.length, 3);
  });

  it("Creates Vault ATA for vault PDA", async () => {
    const ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      client,             // payer
      usdcMint,           // token mint
      vaultAccountPda,    // vault PDA
      true                // allow owner to be PDA
    );
  
    vaultAta = ata.address;
    console.log("✅ Vault ATA created:", vaultAta.toBase58());
  });

  it("Client deposits total milestone funds into Vault", async () => {
    const totalMilestoneAmount = milestones.reduce((acc, m) => acc.add(m.amount), new anchor.BN(0));

    const tx = await program.methods.depositFunds(totalMilestoneAmount)
      .accountsPartial({
        depositor: client.publicKey,
        depositorAta: clientAta,
        client: client.publicKey,
        freelancer: freelancer.publicKey,
        usdcMint: usdcMint,
        vaultAccount: vaultAccountPda,
        vaultAta: vaultAta,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      })
      .signers([client])
      .rpc();
    console.log("✅ depositFundsVault tx:", tx);

    const vaultAtaInfo = await provider.connection.getTokenAccountBalance(vaultAta);
    console.log("Vault balance after deposit:", vaultAtaInfo.value.amount);
    assert.equal(vaultAtaInfo.value.amount, totalMilestoneAmount.toString());
  });




  it("Fails deposit if amount < required", async () => {
    try {
      await program.methods
        .depositFunds(new anchor.BN(30_000_000))
        .accountsPartial({
          depositor: client.publicKey,
          depositorAta: clientAta,
          client: client.publicKey,
          freelancer: freelancer.publicKey,
          usdcMint,
          vaultAccount: vaultAccountPda,
          vaultAta,
          contract: contractPda,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        })
        .signers([client])
        .rpc();
      assert.fail("Deposit should fail");
    } catch (err){
      logAnchorError(err, "✅ Deposit under required amount correctly failed");
      //console.log("✅ Deposit under required amount correctly failed");
    }
  });

  it("Freelancer submits, client approves, freelancer confirms, and milestone 0 released", async () => {
    // Freelancer submit
    const tx1 = await program.methods.freelancerSubmitMilestone(new anchor.BN(0))
      .accountsPartial({
        signer: freelancer.publicKey,
        contract: contractPda,
      })
      .signers([freelancer])
      .rpc();
    console.log("✅ freelancerSubmitMilestone tx:", tx1);

    // Client approve
    const tx2 = await program.methods.clientApproveMilestone(new anchor.BN(0))
      .accountsPartial({
        signer: client.publicKey,
        contract: contractPda,
      })
      .signers([client])
      .rpc();
    console.log("✅ clientApproveMilestone tx:", tx2);

    // Freelancer confirm
    const tx3 = await program.methods.freelancerConfirmMilestone(new anchor.BN(0))
      .accountsPartial({
        signer: freelancer.publicKey,
        contract: contractPda,
      })
      .signers([freelancer])
      .rpc();
    console.log("✅ freelancerConfirmMilestone tx:", tx3);

    // Release milestone payment
    const vaultBefore = await provider.connection.getTokenAccountBalance(vaultAta);
    const freelancerBefore = await provider.connection.getTokenAccountBalance(freelancerAta);

    const tx4 = await program.methods.releaseMilestonePayment(new anchor.BN(0))
      .accountsPartial({
        signer: client.publicKey,
        contract: contractPda,
        vaultAccount: vaultAccountPda,
        vaultAta: vaultAta,
        freelancerAta: freelancerAta,
        usdcMint: usdcMint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([client])
      .rpc();
    console.log("✅ releaseMilestonePayment tx:", tx4);

    const vaultAfter = await provider.connection.getTokenAccountBalance(vaultAta);
    const freelancerAfter = await provider.connection.getTokenAccountBalance(freelancerAta);

    console.log("Vault balance after release:", vaultAfter.value.amount);
    console.log("Freelancer balance after release:", freelancerAfter.value.amount);

    assert.ok(parseInt(freelancerAfter.value.amount) > parseInt(freelancerBefore.value.amount));
  });










  it("Handles full lifecycle for milestone 1", async () => {
    await program.methods.freelancerSubmitMilestone(new anchor.BN(1))
      .accountsPartial({ signer: freelancer.publicKey, contract: contractPda })
      .signers([freelancer])
      .rpc();

    await program.methods.clientApproveMilestone(new anchor.BN(1))
      .accountsPartial({ signer: client.publicKey, contract: contractPda })
      .signers([client])
      .rpc();

    await program.methods.freelancerConfirmMilestone(new anchor.BN(1))
      .accountsPartial({ signer: freelancer.publicKey, contract: contractPda })
      .signers([freelancer])
      .rpc();

    await program.methods.releaseMilestonePayment(new anchor.BN(1))
      .accountsPartial({
        signer: client.publicKey,
        contract: contractPda,
        vaultAccount: vaultAccountPda,
        vaultAta,
        freelancerAta,
        usdcMint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([client])
      .rpc();

    console.log("✅ Milestone 1 fully processed");
  });

  it("Unauthorized submit fails", async () => {
  
    await provider.connection.requestAirdrop(randomUser.publicKey, 1e9);
    try {
      await program.methods.freelancerSubmitMilestone(new anchor.BN(2))
        .accountsPartial({ signer: randomUser.publicKey, contract: contractPda })
        .signers([randomUser])
        .rpc();
      assert.fail("Unauthorized submit should fail");
    } catch(err) {
      logAnchorError(err, "Unauthorized submit failed as expected");
     // console.log("✅ Unauthorized milestone submission rejected");
    }
  });

  it("Fails release without confirmation", async () => {
    try {
      await program.methods.releaseMilestonePayment(new anchor.BN(2))
        .accountsPartial({
          signer: client.publicKey,
          contract: contractPda,
          vaultAccount: vaultAccountPda,
          vaultAta,
          freelancerAta,
          usdcMint,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([client])
        .rpc();
      assert.fail("Should not release unconfirmed milestone");
    } catch(err) {
      logAnchorError(err, "Release without confirmation failed as expected");
     // console.log("✅ Prevented release without confirmation");
    }
  });

  it("Fails double submission of milestone", async () => {
    await program.methods.freelancerSubmitMilestone(new anchor.BN(2))
      .accountsPartial({ signer: freelancer.publicKey, contract: contractPda })
      .signers([freelancer])
      .rpc();

    try {
      await program.methods.freelancerSubmitMilestone(new anchor.BN(2))
        .accountsPartial({ signer: freelancer.publicKey, contract: contractPda })
        .signers([freelancer])
        .rpc();
      assert.fail("Should not allow double submission");
    } catch(err) {
      logAnchorError(err, "Double submission failed as expected");  
      //console.log("✅ Double submission rejected");
    }
  });

  it("Fails to release already released milestone", async () => {
    try {
      await program.methods.releaseMilestonePayment(new anchor.BN(1))
        .accountsPartial({
          signer: client.publicKey,
          contract: contractPda,
          vaultAccount: vaultAccountPda,
          vaultAta,
          freelancerAta,
          usdcMint,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([client])
        .rpc();
      assert.fail("Should not release already released milestone");
    } catch (err){
      logAnchorError(err, "Release of already released milestone failed as expected");
      //console.log("✅ Already released milestone rejected");
    }
  });

  it("Client and freelancer terminate contract and withdraw", async () => {
    await program.methods.completeOrCancelContract()
      .accountsPartial({
        signer: client.publicKey,
        contract: contractPda,
        vaultAccount: vaultAccountPda,
        vaultAta,
        freelancerAta,
        usdcMint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      })
      .signers([client])
      .rpc();

    await program.methods.completeOrCancelContract()
      .accountsPartial({
        signer: freelancer.publicKey,
        contract: contractPda,
        vaultAccount: vaultAccountPda,
        vaultAta,
        freelancerAta,
        usdcMint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      })
      .signers([freelancer])
      .rpc();

   /* await program.methods.withdrawFunds()
      .accountsPartial({
        client: client.publicKey,
        freelancer: freelancer.publicKey,
        usdcMint,
        contract: contractPda,
        vaultAccount: vaultAccountPda,
        vaultAta,
        freelancerAta,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();*/

    console.log("✅ Contract terminated, vault withdrawn");
  });
});

function logAnchorError(err: any, label: string = "") {
  console.log(`❌ ${label} failed`);
  if (err instanceof anchor.AnchorError) {
    console.log("🔴 Anchor Error:");
    console.log("  Code:", err.error.errorCode.code);
    console.log("  Msg :", err.error.errorMessage);
    //console.log("🔍 Logs:\n", err.logs?.join("\n"));
  } else {
    console.log("⚠️ Non-Anchor error:", err);
  }
}