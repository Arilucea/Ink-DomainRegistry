
const { BN, BN_ONE } = require("@polkadot/util");

const MAX_CALL_WEIGHT = new BN(500_000_000_000).isub(BN_ONE);
const PROOFSIZE = new BN(1_000_000);
const storageDepositLimit = null;

function sleep(ms) {
  return new Promise((resolve) => {setTimeout(resolve, ms)});
}

async function callFunction(api, contract, functionName, caller, ...params) {
    let result, output;
    ({result, output} = await contract.query[functionName](
        caller,
        {
        gasLimit: api?.registry.createType('WeightV2', {
            refTime: MAX_CALL_WEIGHT,
            proofSize: PROOFSIZE,
        }),
        storageDepositLimit,
        }, ...params
    ));

    // check if the call was successful
    if (result.isOk) {
    return(output.toHuman()["Ok"]);
    } else {
      return('Error', result.toHuman());
    }
}

async function sendTx(api, contract, functionName, caller, sendValue, ...params) {
  let value = {"error": false};
  await contract.tx[functionName]({
    gasLimit: api?.registry.createType('WeightV2', {
        refTime: MAX_CALL_WEIGHT,
        proofSize: PROOFSIZE,
    }), storageDepositLimit, value: sendValue
    }, ...params).signAndSend(caller, ({ status, events, dispatchError }) => {
    if (dispatchError) {
      if (dispatchError.isModule) {
        const decoded = api.registry.findMetaError(dispatchError.asModule);
        const { docs, name, section } = decoded;
        // console.log(`${section}.${name}: ${docs.join(' ')}`);
        value = {"error": true, "function": functionName};
      } 
    }
  });

  await sleep(500);
  return value
}

module.exports = {
  callFunction,
  sendTx,
  sleep,
}
