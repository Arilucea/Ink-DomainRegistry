var assert = require('assert');

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { ContractPromise } = require('@polkadot/api-contract');

const registry_metada = require('./contract-files/metadata.json');
const {callFunction, sendTx} = require('./utils.js');

const CONTRACT = '5Chwf9pJYHLRJT8qCpdA8HLMQevwe7vtWgeBzGtgLrGNbg7R';

describe('Domain registry test', async function () {
    let wsProvider;
    let api;
    let contract;

    let testDomain = 'testDomain2';
    let rentDuration = 30000000000;
    let rentPriceLetter = 500;

    const keyring = new Keyring({ type: 'sr25519' });
    let alice;
    let bob;

    before(async () => {
        wsProvider = new WsProvider('ws://127.0.0.1:9944');
        api = await ApiPromise.create({ provider: wsProvider });
        await api.isReady;
        contract = new ContractPromise(api, registry_metada, CONTRACT);
        alice = keyring.addFromUri('//Alice');    
        bob = keyring.addFromUri('//Bob');    
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
        assert.equal(parseInt(result), (testDomain.length * rentDuration * rentPriceLetter), 'Domain price not correct');
    });

    it('should be possible to rent a domain', async () => {
        let secret = await callFunction(api, contract, 'generateSecret', alice.address, testDomain, '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSD');
        let result = await sendTx(api, contract, 'requestDomain', alice, 0, secret);
        assert.equal(result.error, false, `Tx failed function ${result.function}`);
        
        let domainPrice = (await callFunction(api, contract, 'rentPrice', alice.address, testDomain, rentDuration)).replaceAll(',', '');
        result = await sendTx(api, contract, 'rentDomain', alice, domainPrice, testDomain, '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSD', rentDuration, 'test domain metadata');
        assert.equal(result.error, false, `Tx failed function ${result.function}`);

        let domainData = await callFunction(api, contract, 'getDomainData', alice.address, testDomain);
        assert.equal(domainData.owner, alice.address, 'Owner not correct');
        assert.equal(domainData.metadata, 'test domain metadata', 'Metadata not correct');
    });

    it('should be possible to renew a domain', async () => {
        let initialDomainData = await callFunction(api, contract, 'getDomainData', alice.address, testDomain);
        let result = await sendTx(api, contract, 'renewDomain', alice, 2000000000000000, testDomain, 7000000);
        assert.equal(result.error, false, `Tx failed function ${result.function}`);
        let renewDomainData = await callFunction(api, contract, 'getDomainData', alice.address, testDomain);
        assert.notEqual(initialDomainData.expirationDate, renewDomainData.expirationDate, 'Expiration date has not changed');
    });

    it('should be possible to refund an expired domain', async () => {
        let secret = await callFunction(api, contract, 'generateSecret', alice.address, 'testDomainRefund', '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSD');
        let result = await sendTx(api, contract, 'requestDomain', alice, 0, secret);
        assert.equal(result.error, false, `Tx failed function ${result.function}`);
        result = await sendTx(api, contract, 'updateMinLockTime', alice, 0, 1);
        assert.equal(result.error, false, `Tx failed function ${result.function}`);

        domainPrice = (await callFunction(api, contract, 'rentPrice', alice.address, 'testDomainRefund', 1)).replaceAll(',', '');
        let accountDataBefore  = await api.query.system.account(CONTRACT);
        result = await sendTx(api, contract, 'rentDomain', alice, domainPrice, 'testDomainRefund', '5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSD', 1, 'test domain metadata');
        assert.equal(result.error, false, `Tx failed function ${result.function}`);

        let accountDataRented  = await api.query.system.account(CONTRACT);
        assert.notEqual((accountDataBefore.data.free).toNumber(), (accountDataRented.data.free).toNumber(), `Contract balance is not correct, has not increased on rent`);

        result = await sendTx(api, contract, 'refundDomain', alice, 0, 'testDomainRefund');
        assert.equal(result.error, false, `Tx failed function ${result.function}`);
        let accountDataAfter  = await api.query.system.account(CONTRACT);
        assert.equal((accountDataBefore.data.free).toNumber(), (accountDataAfter.data.free).toNumber(), `Contract balance is not correct, fee has not been returned to the domain owner`);
    });

    it('should not be possible to update the lock time by a user that is not the owner of the contract', async () => {
        let result = await sendTx(api, contract, 'updateMinLockTime', bob, 0, 1);
        assert.equal(result.error, true, `Tx failed function ${result.function}`);
    });

});
