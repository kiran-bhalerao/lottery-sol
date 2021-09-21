const path = require("path");
const fs = require("fs");
const os = require("os");
const yaml = require("yaml");
const { Account } = require("@solana/web3.js");
const Lottery = require("./lottery");

async function getConfig() {
  // Path to Solana CLI config file
  const CONFIG_FILE_PATH = path.resolve(
    os.homedir(),
    ".config",
    "solana",
    "cli",
    "config.yml"
  );
  const configYml = fs.readFileSync(CONFIG_FILE_PATH, { encoding: "utf8" });
  return yaml.parse(configYml);
}

async function loadProgramAddress() {
  const dir = path.join(__dirname, "../dist/main-keypair.json");
  try {
    const programKey = require(dir);
    const program = new Account(programKey);
    return program.publicKey.toBase58();
  } catch (er) {
    throw new Error("You must build and deploy the program first");
  }
}

async function loadPayer() {
  const config = await getConfig();
  if (!config.keypair_path) throw new Error("Missing keypair path");

  try {
    const payerKey = require(config.keypair_path);

    const payer = new Account(payerKey);
    return payer;
  } catch (er) {
    throw new Error("You must create a payer account first");
  }
}

// create new Lottery account or use existing one
async function loadLotteryAddress(lotteryInstance, payer) {
  const dir = path.join(__dirname, "../dist/lottery-keypair.json");
  try {
    // Load the existed account
    const helloKey = require(dir);
    const account = new Account(helloKey);
    return account.publicKey.toBase58();
  } catch (er) {
    // Create a new one
    const account = await lotteryInstance.rentHelloAccount(payer);
    // Store it
    const data = "[" + account.secretKey.toString() + "]";
    fs.writeFileSync(dir, data, "utf8");
    return account.publicKey.toBase58();
  }
}

/**
 * Main
 */
(async () => {
  try {
    const config = await getConfig();
    const payer = await loadPayer();
    const programAddress = loadProgramAddress();

    const lottery = new Lottery(programAddress, config.json_rpc_url);
    console.log("*** Calling to program:", programAddress);
    console.log("*** Payer:", payer.publicKey.toBase58());

    // Build account to store the Lottery data
    const lotteryAddress = await loadLotteryAddress(lottery, payer);

    // Get account data
    const data = await lottery.getLottery(lotteryAddress);
    console.log("Account Data:", data);

    // await lottery.initLottery(lotteryAddress, payer);

    // await lottery.participateLottery(lotteryAddress, payer);

    // change participants pubkey (3rd and 4rth arg)
    // const tx = await lottery.pickWinnerLottery(
    //   lotteryAddress,
    //   payer,
    //   new PublicKey("E73MCtPHkySMX9KmZNBTzBbV8viBtcipxkXdy2cs6wet"),
    //   new PublicKey("HtTv1zuyyDCbvTfW1HezRpXL24LixKgrLxx1VUddXdoB")
    // );

    // if (tx) {
    //   // pickWinner will delete lottery account so remove the lottery account pubkey file
    //   const dir = path.join(__dirname, "../dist/lottery-keypair.json");
    //   fs.unlinkSync(dir);
    // }
  } catch (er) {
    return console.error(er);
  }
})();
