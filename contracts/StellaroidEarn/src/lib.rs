#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, BytesN, Env,
};

// ── Storage key ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Certificate(BytesN<32>),
}

// ── Certificate data ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct Certificate {
    pub owner:    Address,
    pub verified: bool,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct StellaroidEarn;

#[contractimpl]
impl StellaroidEarn {

    /// Register a certificate with its hash and owner wallet.
    /// Panics if the certificate already exists (duplicate detection).
    pub fn register_certificate(env: Env, hash: BytesN<32>, owner: Address) {
        owner.require_auth();

        let key = DataKey::Certificate(hash.clone());

        if env.storage().persistent().has(&key) {
            panic!("certificate already exists");
        }

        let cert = Certificate {
            owner,
            verified: true,
        };

        env.storage().persistent().set(&key, &cert);

        env.events().publish(
            (symbol_short!("cert"), symbol_short!("issued")),
            hash,
        );
    }

    /// Transfer XLM reward from the contract to the verified student wallet.
    /// The contract must be funded with XLM before calling this.
    pub fn reward_student(
        env:       Env,
        hash:      BytesN<32>,
        amount:    i128,
        xlm_token: Address,
    ) {
        let cert: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(hash.clone()))
            .expect("certificate not found");

        if !cert.verified {
            panic!("certificate not verified");
        }

        let token_client = token::Client::new(&env, &xlm_token);
        token_client.transfer(
            &env.current_contract_address(),
            &cert.owner,
            &amount,
        );

        env.events().publish(
            (symbol_short!("reward"), symbol_short!("paid")),
            (hash, amount),
        );
    }

    /// Verify a certificate — returns true/false and emits an on-chain event.
    pub fn verify_certificate(env: Env, hash: BytesN<32>) -> bool {
        let cert: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(hash.clone()))
            .expect("certificate not found");

        env.events().publish(
            (symbol_short!("verify"), symbol_short!("cert")),
            (hash, cert.verified),
        );

        cert.verified
    }

    /// Employer triggers a direct XLM payment to the verified student wallet.
    pub fn link_payment(
        env:       Env,
        hash:      BytesN<32>,
        from:      Address,
        amount:    i128,
        xlm_token: Address,
    ) {
        from.require_auth();

        let cert: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(hash.clone()))
            .expect("certificate not found");

        if !cert.verified {
            panic!("certificate not verified");
        }

        let token_client = token::Client::new(&env, &xlm_token);
        token_client.transfer(&from, &cert.owner, &amount);

        env.events().publish(
            (symbol_short!("payment"), symbol_short!("sent")),
            (hash, amount),
        );
    }

    /// Returns the certificate owner address.
    pub fn get_owner(env: Env, hash: BytesN<32>) -> Address {
        let cert: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(hash))
            .expect("certificate not found");
        cert.owner
    }

    /// Returns true if the certificate exists and is verified.
    pub fn is_verified(env: Env, hash: BytesN<32>) -> bool {
        match env
            .storage()
            .persistent()
            .get::<DataKey, Certificate>(&DataKey::Certificate(hash))
        {
            Some(cert) => cert.verified,
            None => false,
        }
    }
}