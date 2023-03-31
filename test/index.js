// Import the API
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { ContractPromise } = require('@polkadot/api-contract');

const registry_metada = require('./contract-files/metadata.json');
const {callGetFunction} = require('./utils.js');

// Our address for Alice on the dev chain
const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const BOB = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

const CONTRACT = '5GW3kKWdy96gPz5TLRw59CvkDNCGQwNpH8akW82kv4PdseA4';

async function main () {
  // Create our API with a default connection to the local node
  const wsProvider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider: wsProvider });
  // Wait until we are ready and connected
  await api.isReady;

  // Retrieve the last block header, extracting the hash and parentHash
  const { hash, parentHash } = await api.rpc.chain.getHeader();

  console.log(`last header hash ${hash.toHex()}`);

  // Retrieve the balance at the preceding block for Alice using an at api
  const apiAt = await api.at(parentHash);
  const balance = await apiAt.query.system.account(ALICE);

  console.log(`Alice's balance at ${parentHash.toHex()} was ${balance.data.free}`);

  // Now perform a multi query, returning multiple balances at once
  const balances = await api.query.system.account.multi([ALICE, BOB]);

  console.log(`Current balances for Alice and Bob are ${balances[0].data.free} and ${balances[1].data.free}`);

  // The address is the actual on-chain address as ss58 or AccountId object.
  const contract = new ContractPromise(api, registry_metada, CONTRACT);

  // if null is passed, unlimited balance can be used
  
  let result;

  result = await callGetFunction(api, contract, "getDomainData", ALICE, "test");
  console.log(result)
  // result = await callGetFunction(api, contract, "get", ALICE);
  console.log(result)
  result = await callGetFunction(api, contract, "getDomainData", BOB, "othertest");
  console.log(result)
  result = await callGetFunction(api, contract, "rentPrice", BOB, "othertest", 10000000000);
  console.log(result)
}

main().catch(console.error).finally(() => process.exit());