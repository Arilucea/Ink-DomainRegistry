var assert = require('assert');

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { ContractPromise } = require('@polkadot/api-contract');

const registry_metada = require('./contract-files/metadata.json');
const {callFunction} = require('./utils.js');

// Our address for Alice on the dev chain
const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const BOB = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

const CONTRACT = '5DLpUUmVWgufc9a9v4D7zuqFnCKJkERBo57txM8CYL5BL5cv';

describe('Domain registry test', async function () {

    let wsProvider;
    let api;
    let contract;

    let testDomain = 'testDomain';
    let rentDuration = 30000000000;

    before(async () => {
        wsProvider = new WsProvider('ws://127.0.0.1:9944');
        api = await ApiPromise.create({ provider: wsProvider });
        // Wait until we are ready and connected
        await api.isReady;
        contract = new ContractPromise(api, registry_metada, CONTRACT);
    });

    after( async () => {
        await wsProvider.disconnect();
    });

    it('should return empty domain data if the domain is not registered', async () => {
        result = await callFunction(api, contract, 'getDomainData', ALICE, testDomain);
        assert.equal(result.owner, '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM', 'Owner not correct');
        assert.equal(result.expirationDate, 0, 'Expiration date not correct');
        assert.equal(result.metadata, '', 'Metadata not correct');
    });

    it('should return the domain price', async () => {
        result = (await callFunction(api, contract, 'rentPrice', BOB, testDomain, rentDuration)).replaceAll(',', '');
        assert.equal(parseInt(result), (testDomain.length * rentDuration), 'Domain price not correct');
    });
});
