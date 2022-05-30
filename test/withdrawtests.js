const nearAPI = require("near-api-js");
const fs = require('fs');
// creates keyStore from a private key string
// you can define your key here or use an environment variable

// creates keyStore from a private key string
// you can define your key here or use an environment variable

const { keyStores, KeyPair } = nearAPI;
const keyStore = new keyStores.InMemoryKeyStore();
const PRIVATE_KEY =
    "4GrZBkRSEp8YT6ztXHUu9wzrDb3qrpFpyTzEsFR5yovjbGqt16aKQVR7WHoMUdBoNwe2NJRGZ22mt1o3j2wda1jk";
const SENDER_ADDRESS = "cuongcute.testnet";
// creates a public / private key pair using the provided private key
const keyPair = KeyPair.fromString(PRIVATE_KEY);
console.log({keyPair});
const { connect } = nearAPI;

(async () => {
    // adds the keyPair you created to keyStore
    await keyStore.setKey("testnet", SENDER_ADDRESS, keyPair);

    const config = {
        networkId: "testnet",
        keyStore,
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
    };
    const near = await connect(config);
    const senderAccount = await near.account(SENDER_ADDRESS)

    

    console.log({senderAddress: senderAccount});

    const contractId = "496add2c24e17711d9512172901b5502df37e10493d247c371eb8dc3e4b173fc";
    console.log(contractId);

    const contract = new nearAPI.Contract(
        senderAccount, // account object đang kết nối
        contractId,
        {
          changeMethods: ["withdraw"], 
          sender: senderAccount,
        }
      );

    unshieldInfo = {
        inst: "9d01280000000000000000000000000000000000000000000000003030303030303030303030303030303030303030303030303030303030303030303030303030303111000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063756f6e67637574652e746573746e657400000000000000000000000000000000000000000000000000000000000f4240fa18f0a998d51ae514c97bcf158da0273611949d596aac100e6e6adcff321cdd5508e0f98d4ad4b72e06600dc3e9e09f6e5f8baf8b84a46e168d0ac42dd4d8ff",
        height: 26,
        inst_paths: [
            to32Bytes("abdc7388849a767a7e2158b57514c56e8e0e0e0c67bf64a67754d027b70b4c65"),
            to32Bytes("2039b1984afbf2b42f45d86c7125a9a27cd3dc4950b2a4a91f2245f26a97f42c"),

        ],
        inst_path_is_lefts: [
            false,
            false,
        ],
        inst_root: to32Bytes("6933e4b974053b95c561a1e48a23028a3ba5ccbda927cddbbe68c84435beaa83"),
        blk_data: to32Bytes("a6425287b73390192266da5ceaf04430bc6925eef5d137cff183043d04ab7e3e"),
        indexes: [0, 1, 2, 3],
        signatures: [
            "2b1ff1b9db10ec64acc562eeab8e11e5f18ae00cb283908ee333692510ee6dc5030bdb7cc594d6ff93c1b3c133ea543679be397d348d2f55de7f15e51631e2c400",
            "d8b4382b92e36d6e437f65e93174abd01cd4fa063a60dbcd74650f0cca46c7b439110068ab73198fc995b362c376d1684f5b46994042e210af7d159917ad108a01",
            "f55e76f39c25295800600937eb37bd236c216165cbf147afa4555ce96de30daf2a690cd2676541f679ee691261220f6cc74f46231d3e9dd3468c6b642ddcf42100",
            "79b62a19a6ced649fda27daa8fc832f4ff1423e7f451c623cfc56e31c5ce321c20233f50ef786ab25effb2bc755324a8ad8c192e12967f2707df911aa9b62aa700",
        ],
        vs: [0, 1, 0, 0],
    },
    console.log({unshieldInfo: unshieldInfo});
      
    await contract.withdraw(
            {
                unshield_info: unshieldInfo,
            },
            "300000000000000",
            "0"
    );

})();

function to32Bytes(hexStr) {
    const bytes = Buffer.from(hexStr, "hex");
    const padded = Buffer.alloc(32);
    bytes.copy(padded);
    const arr = [...padded];
    return arr;
}