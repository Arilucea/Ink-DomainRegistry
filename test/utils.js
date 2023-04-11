
const { BN, BN_ONE } = require("@polkadot/util");

const MAX_CALL_WEIGHT = new BN(5_000_000_000_000).isub(BN_ONE);
const PROOFSIZE = new BN(1_000_000);
const storageDepositLimit = null

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

module.exports = {
  callFunction,
}
