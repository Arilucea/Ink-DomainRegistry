var assert = require('assert');

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { ContractPromise } = require('@polkadot/api-contract');

const registry_metada = require('./contract-files/metadata.json');
const {callFunction, sendTx} = require('./utils.js');

const CONTRACT = '5HbbZB7mBpLJggRRgy4JXbYqAwat4sGjcRZYGJyB8xknyB6p';

describe('Domain registry test', async function () {

    let wsProvider;
    let api;
    let contract;

    let testDomain = 'testDomain99';
    let rentDuration = 30000000000;

    const keyring = new Keyring({ type: 'sr25519' });
    let alice;

    before(async () => {
        wsProvider = new WsProvider('ws://127.0.0.1:9944');
        api = await ApiPromise.create({ provider: wsProvider });
        await api.isReady;
        contract = new ContractPromise(api, registry_metada, CONTRACT);
        alice = keyring.addFromUri('//Alice');    
    });

    after( async () => {
        await wsProvider.disconnect();
    });

    it('should return empty domain data if the domain is not registered', async () => {
        domainData = await callFunction(api, contract, 'getDomainData', alice.address, 'emptyDomain');
        assert.equal(domainData.owner, '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM', 'Owner not correct');
        assert.equal(domainData.expirationDate, 0, 'Expiration date not correct');
        assert.equal(domainData.metadata, '', 'Metadata not correct');
    });

    it('should return the domain price', async () => {
        result = (await callFunction(api, contract, 'rentPrice', alice.address, testDomain, rentDuration)).replaceAll(',', '');
        assert.equal(parseInt(result), (testDomain.length * rentDuration), 'Domain price not correct');
    });

    it('should be possible to rent a domain', async () => {
        let secret = await callFunction(api, contract, 'generateSecret', alice.address, testDomain, '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSD');
        await sendTx(api, contract, 'requestDomain', alice, 0, secret);
        let domainPrice = (await callFunction(api, contract, 'rentPrice', alice.address, testDomain, rentDuration)).replaceAll(',', '');
        await sendTx(api, contract, 'rentDomain', alice, domainPrice, testDomain, '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSD', rentDuration, 'test domain metadata');

        let domainData = await callFunction(api, contract, 'getDomainData', alice.address, testDomain);
        assert.equal(domainData.owner, alice.address, 'Owner not correct');
        assert.equal(domainData.metadata, 'test domain metadata', 'Metadata not correct');
    });

    it('should be possible to renew a domain', async () => {
        let initialDomainData = await callFunction(api, contract, 'getDomainData', alice.address, testDomain);
        await sendTx(api, contract, 'renewDomain', alice, 200000000000000, testDomain, 70000000000);
        let renewDomainData = await callFunction(api, contract, 'getDomainData', alice.address, testDomain);
        assert.notEqual(initialDomainData.expirationDate, renewDomainData.expirationDate, 'Expiration date has not changed');
    });

});
