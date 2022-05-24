const nearAPI = require("near-api-js");
const fs = require('fs');
// creates keyStore from a private key string
// you can define your key here or use an environment variable

// creates keyStore from a private key string
// you can define your key here or use an environment variable

const { keyStores, KeyPair } = nearAPI;
const keyStore = new keyStores.InMemoryKeyStore();
const PRIVATE_KEY =
    "2wkhjNeyKBWnaeCPtrktGMGU4UGQqPKx59gpaWGPoUUPQLzwqdSPeAfF5ABMiy16PVrM3xP7icecpheYNsJa4R7h";
// creates a public / private key pair using the provided private key
const keyPair = KeyPair.fromString(PRIVATE_KEY);
console.log({keyPair});
const { connect } = nearAPI;

(async () => {
    const pk58 = 'ed25519:3PSXhTci6Fyh6TmuZH5to8apvtMxw3kqanBbVAhAUcEf'
    const testAddress = nearAPI.utils.PublicKey.fromString(pk58).data.hexSlice();

    // adds the keyPair you created to keyStore
    await keyStore.setKey("testnet", testAddress, keyPair);
    const config = {
        networkId: "testnet",
        keyStore,
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
    };
    const near = await connect(config);
    const account = await near.account(testAddress);
    console.log({testAddress});
    // const account = await near.account("incognito.bridge.testnet");
    // // await account.createAccount(
    // //     "example-account2.testnet", // new account name
    // //     "8hSHprDq2StXwMtNd43wDTXQYsjXcD4MJTXQYsjXcc", // public key for new account
    // //     "10000000000000000000" // initial balance for new account in yoctoNEAR
    // // );

    let balance = await account.getAccountBalance();
    console.log({balance});

    // const response = await account.deployContract(fs.readFileSync('../target/wasm32-unknown-unknown/release/bridge.wasm'));
    // console.log(response);
    // const contractId = response.transaction_outcome.outcome.executor_id;
    const contractId = "23770487361340580c3aea1a0a74fd3048aabd90746faafa4238c597b1dc280c";
    console.log(contractId);

    const beacon1 = toHexString([64,206,253,84,56,206,63,162,157,152,148,80,198,23,66,245,43,1,207,238,9,144,161,139,131,44,146,136,74,242,22,220,187,130,145,153,93,114,117,199,108,190,233,244,53,240,247,48,207,19,94,245,14,171,207,124,157,177,173,139,253,237,36,168]);
    const beacon2 = toHexString([175,109,126,18,52,108,137,78,38,252,216,214,224,214,44,187,2,67,70,204,196,78,155,224,72,126,124,128,134,165,210,158,138,93,62,90,76,225,186,39,215,204,170,10,127,99,86,220,107,251,34,58,235,236,69,189,235,226,57,208,106,210,28,22]);
    const beacon3 = toHexString([122,69,179,100,37,117,17,36,0,4,211,125,150,102,106,180,218,127,238,200,104,84,250,183,23,31,209,229,22,117,248,73,56,120,112,2,188,187,152,44,70,228,25,160,250,255,40,216,180,239,183,235,175,79,66,41,119,82,195,70,103,102,135,73]);
    const beacon4 = toHexString([24,171,11,173,118,80,213,52,20,186,77,213,182,249,188,70,15,37,228,129,102,45,183,139,139,174,147,32,130,179,168,171,36,79,30,237,44,11,200,229,108,224,117,224,206,11,62,235,127,101,194,116,209,213,122,41,77,229,19,60,199,168,81,25]);

    const contract = new nearAPI.Contract(
        account, // the account object that is connecting
        contractId,
        {
            // name of contract you're connecting to
            viewMethods: ["get_beacons", "get_tx_burn_used"], // view methods do not change state but usually return a value
            changeMethods: ["new", "deposit", "withdraw", "swap_beacon_committee", "submit_burn_proof"], // change methods modify state
            sender: account, // account object to initialize and sign transactions.
        }
    );

    // init bridge contract
    // await contract.new(
    //     {
    //         args: {
    //             beacons: [
    //                 beacon1,
    //                 beacon2,
    //                 beacon3,
    //                 beacon4
    //             ],
    //             height: 0,
    //         },
    //         gas: "300000000000000",
    //         amount: "0"
    //     },
    // );

    const beaconlist = await contract.get_beacons({
        height: 0
    });
    console.log({beaconlist});

    // make shield Near request
    const incognitoAddress = "12sfD6DYsmYFGvZHbkmVhQiKyapWwshxKtMDZV51UFpXwaauCZ7Zyp69ctAQo3BJdKpZeZhVkFfCd8BgT6n4sMuRAhszpJ6pbwXct3Mr5kvCzDEgBz7h9mgoGuqwt83CjLCDuX7b7hP6gf9RWuPb";
    await contract.deposit(
        {
            args: {
                incognito_address: incognitoAddress
            },
            gas: "300000000000000",
            amount: "1000000000000000000000"
        },
    );

})();

function toHexString(byteArray) {
    return Array.from(byteArray, function(byte) {
        return ('0' + (byte & 0xFF).toString(16)).slice(-2);
    }).join('')
}