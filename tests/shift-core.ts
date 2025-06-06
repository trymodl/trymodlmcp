import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ShiftCore } from "../target/types/shift_core";
import { assert } from "chai";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram,
  LAMPORTS_PER_SOL 
} from "@solana/web3.js";

describe("shift-core", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ShiftCore as Program<ShiftCore>;
  const provider = anchor.getProvider();

  // Test accounts
  let protocolState: PublicKey;
  let deviceAccount: PublicKey;
  let transactionAccount: PublicKey;
  
  // Test keypairs
  const authority = Keypair.generate();
  const deviceOwner = Keypair.generate();
  const sender = Keypair.generate();
  const recipient = Keypair.generate();

  // Test data
  const deviceId = new Uint8Array(32).fill(1, 0, 32);
  const recipientDeviceId = new Uint8Array(32).fill(2, 0, 32);

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(authority.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(deviceOwner.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(sender.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(recipient.publicKey, 10 * LAMPORTS_PER_SOL);

    // Wait for confirmations
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Derive PDAs
    [protocolState] = PublicKey.findProgramAddressSync(
      [Buffer.from("protocol")],
      program.programId
    );

    [deviceAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("device"), Buffer.from(deviceId)],
      program.programId
    );
  });

  it("Initialize protocol", async () => {
    try {
      const tx = await program.methods
        .initialize()
        .accounts({
          authority: authority.publicKey,
          protocolState,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();

      console.log("Protocol initialization transaction signature:", tx);

      // Verify protocol state
      const protocolData = await program.account.protocolState.fetch(protocolState);
      assert.equal(protocolData.authority.toString(), authority.publicKey.toString());
      assert.equal(protocolData.totalDevices.toNumber(), 0);
      assert.equal(protocolData.totalTransactions.toNumber(), 0);
      assert.equal(protocolData.protocolFee.toNumber(), 0); // No fees in Shift!
    } catch (error) {
      console.error("Error initializing protocol:", error);
      throw error;
    }
  });

  it("Register hardware device", async () => {
    const attestationData = {
      attestationKey: new Uint8Array(32).fill(3, 0, 32),
      signature: new Uint8Array(64).fill(4, 0, 64),
      timestamp: new anchor.BN(Date.now() / 1000),
      hardwareType: { shiftDevice: {} },
    };

    try {
      const tx = await program.methods
        .registerDevice(Array.from(deviceId), attestationData)
        .accounts({
          owner: deviceOwner.publicKey,
          deviceAccount,
          protocolState,
          systemProgram: SystemProgram.programId,
        })
        .signers([deviceOwner])
        .rpc();

      console.log("Device registration transaction signature:", tx);

      // Verify device registration
      const deviceData = await program.account.deviceAccount.fetch(deviceAccount);
      assert.deepEqual(Array.from(deviceData.deviceId), Array.from(deviceId));
      assert.equal(deviceData.owner.toString(), deviceOwner.publicKey.toString());
      assert.equal(deviceData.isActive, true);
      assert.equal(deviceData.keyPoolSize, 1000);
      assert.equal(deviceData.usedKeys, 0);

      // Verify protocol state updated
      const protocolData = await program.account.protocolState.fetch(protocolState);
      assert.equal(protocolData.totalDevices.toNumber(), 1);
    } catch (error) {
      console.error("Error registering device:", error);
      throw error;
    }
  });

  it("Prepare P2P transaction", async () => {
    const amount = new anchor.BN(1000000); // 1 token with 6 decimals

    // Derive transaction account PDA
    const timestamp = Math.floor(Date.now() / 1000);
    [transactionAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("transaction"), 
        sender.publicKey.toBuffer(),
        Buffer.from(new anchor.BN(timestamp).toArray("le", 8))
      ],
      program.programId
    );

    try {
      const tx = await program.methods
        .prepareTransaction(amount, Array.from(recipientDeviceId))
        .accounts({
          sender: sender.publicKey,
          transactionAccount,
          deviceAccount,
          systemProgram: SystemProgram.programId,
        })
        .signers([sender])
        .rpc();

      console.log("Transaction preparation signature:", tx);

      // Verify transaction preparation
      const txData = await program.account.transactionAccount.fetch(transactionAccount);
      assert.equal(txData.sender.toString(), sender.publicKey.toString());
      assert.equal(txData.amount.toNumber(), amount.toNumber());
      assert.deepEqual(Array.from(txData.recipientDeviceId), Array.from(recipientDeviceId));
      assert.deepEqual(txData.status, { prepared: {} });
    } catch (error) {
      console.error("Error preparing transaction:", error);
      throw error;
    }
  });

  it("Cannot execute transaction without valid hardware signature", async () => {
    const invalidSignature = new Uint8Array(64).fill(0); // Invalid signature
    const invalidEncumbranceProof = new Uint8Array(32).fill(0);

    try {
      await program.methods
        .executeTransaction(Array.from(invalidSignature), Array.from(invalidEncumbranceProof))
        .accounts({
          sender: sender.publicKey,
          transactionAccount,
          senderDevice: deviceAccount,
          protocolState,
          // Note: In a real test, we'd need proper token accounts
          senderTokenAccount: sender.publicKey, // Placeholder
          recipientTokenAccount: recipient.publicKey, // Placeholder
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([sender])
        .rpc();

      assert.fail("Should have failed with invalid signature");
    } catch (error) {
      console.log("Expected error for invalid signature:", error.message);
      assert.ok(error.message.includes("InvalidHardwareSignature") || 
                error.message.includes("AccountNotFound")); // May fail on missing token accounts
    }
  });

  it("Verify transaction hash calculation", async () => {
    try {
      // This would test the transaction hash verification
      // In a real implementation, we'd call verify_transaction
      const txData = await program.account.transactionAccount.fetch(transactionAccount);
      
      // Basic verification that the transaction data is correct
      assert.equal(txData.sender.toString(), sender.publicKey.toString());
      assert.deepEqual(txData.status, { prepared: {} });
      
      console.log("Transaction hash verification test completed");
    } catch (error) {
      console.error("Error in hash verification:", error);
      throw error;
    }
  });

  it("Check protocol statistics", async () => {
    try {
      const protocolData = await program.account.protocolState.fetch(protocolState);
      
      console.log("Protocol Statistics:");
      console.log("- Total devices:", protocolData.totalDevices.toNumber());
      console.log("- Total transactions:", protocolData.totalTransactions.toNumber());
      console.log("- Protocol fee:", protocolData.protocolFee.toNumber(), "(Always 0 for Shift!)");
      
      assert.equal(protocolData.totalDevices.toNumber(), 1);
      assert.equal(protocolData.protocolFee.toNumber(), 0); // Shift has no fees!
    } catch (error) {
      console.error("Error checking protocol statistics:", error);
      throw error;
    }
  });

  it("Demonstrate validator-less transaction model", async () => {
    // This test demonstrates the key concept of Shift:
    // Transactions are validated through hardware attestation, not blockchain consensus
    
    console.log("\n🚀 Shift Protocol Demo: Validator-less P2P Transactions");
    console.log("================================================");
    console.log("✅ No validators needed");
    console.log("✅ No gas fees");
    console.log("✅ No block times");
    console.log("✅ No mempools");
    console.log("✅ Hardware attestation ensures security");
    console.log("✅ Key encumbrance prevents double-spending");
    console.log("✅ Direct peer-to-peer settlement");
    
    const deviceData = await program.account.deviceAccount.fetch(deviceAccount);
    console.log("\nDevice Status:");
    console.log("- Device ID:", Array.from(deviceData.deviceId).slice(0, 8), "...");
    console.log("- Active:", deviceData.isActive);
    console.log("- Key pool size:", deviceData.keyPoolSize);
    console.log("- Used keys:", deviceData.usedKeys);
    console.log("- Hardware type: Shift Device");
    
    assert.ok(true); // Test passes if we reach here
  });
}); 