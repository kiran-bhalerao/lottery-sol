const {
  SystemProgram,
  Connection,
  sendAndConfirmTransaction,
  TransactionInstruction,
  Transaction,
  PublicKey,
  Account,
} = require("@solana/web3.js");

const soproxABI = require("soprox-abi");

// Define maximum number of participants, if u change this var, also change on rust code
const MAX_PARTICIPANT = 2;

const LOTTERY_ACCOUNT_SCHEMA = [
  {
    key: "entry_fees",
    type: "u32",
  },
  {
    key: "commission_rate",
    type: "u8",
  },
  {
    key: "initializer",
    type: "[u8;32]",
  },
  {
    key: "participants",
    type: `[u8;${32 * MAX_PARTICIPANT}]`,
  },
];

class Lottery {
  constructor(programAddress, nodeUrl) {
    this.programId = new PublicKey(programAddress);
    this.connection = new Connection(nodeUrl, "recent");
  }

  async rentLotteryAccount(payer) {
    const account = new Account();
    // Compute needed space for a Lottery account
    const layout = new soproxABI.struct(LOTTERY_ACCOUNT_SCHEMA);
    const space = layout.space;
    // Compute rental fee
    const lamports = await this.connection.getMinimumBalanceForRentExemption(
      space
    );
    const instruction = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: account.publicKey,
      lamports,
      space,
      programId: this.programId,
    });
    const transaction = new Transaction();
    transaction.add(instruction);
    // Send transaction
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [payer, account],
      { skipPreflight: true, commitment: "recent" }
    );
    return account;
  }

  async getLottery(lotteryAddress) {
    const lotteryPublicKey = new PublicKey(lotteryAddress);
    // Get raw data
    const res = await this.connection.getAccountInfo(lotteryPublicKey);
    const { data } = res;
    // Parse data to json
    const layout = new soproxABI.struct(LOTTERY_ACCOUNT_SCHEMA);
    layout.fromBuffer(data);
    // Return the result
    return layout.value;
  }

  async sendTransaction(instruction, payer) {
    const transaction = new Transaction();
    transaction.add(instruction);

    // Send transaction
    const txId = await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [payer],
      { skipPreflight: true, commitment: "recent" }
    );
    return txId;
  }

  async initLottery(helloAddress, payer) {
    const helloPublicKey = new PublicKey(helloAddress);
    // Build input
    const layout = new soproxABI.struct(
      [
        { key: "tag", type: "u8" },
        { key: "entry_fees", type: "u32" },
        { key: "commission_rate", type: "u8" },
      ],
      { tag: 0, entry_fees: 1, commission_rate: 10 }
    );

    const data = layout.toBuffer();

    // Build transaction
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: helloPublicKey, isSigner: false, isWritable: true },
      ],
      programId: this.programId,
      data,
    });

    return this.sendTransaction(instruction, payer);
  }

  async participateLottery(lotteryAddress, payer) {
    const lotteryPublicKey = new PublicKey(lotteryAddress);
    // Build input
    const layout = new soproxABI.struct([{ key: "tag", type: "u8" }], {
      tag: 1,
    });

    const data = layout.toBuffer();

    // Build transaction
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: lotteryPublicKey, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });

    return this.sendTransaction(instruction, payer);
  }

  async pickWinnerLottery(
    lotteryAddress,
    payer,
    user1PublicKey,
    user2PublicKey
  ) {
    const lotteryPublicKey = new PublicKey(lotteryAddress);
    // Build input
    const layout = new soproxABI.struct([{ key: "tag", type: "u8" }], {
      tag: 2,
    });

    const data = layout.toBuffer();

    // Build transaction
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: lotteryPublicKey, isSigner: false, isWritable: true },
        { pubkey: user1PublicKey, isSigner: false, isWritable: true },
        { pubkey: user2PublicKey, isSigner: false, isWritable: true },
      ],
      programId: this.programId,
      data,
    });

    return this.sendTransaction(instruction, payer);
  }
}

module.exports = Lottery;
