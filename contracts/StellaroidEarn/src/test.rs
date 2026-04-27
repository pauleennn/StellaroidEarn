#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::Address as _,
        token, Address, BytesN, Env,
    };
    use crate::{StellaroidEarn, StellaroidEarnClient};

    fn setup() -> (Env, StellaroidEarnClient<'static>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, StellaroidEarn);
        let client = StellaroidEarnClient::new(&env, &contract_id);

        let admin   = Address::generate(&env);
        let student = Address::generate(&env);
        let xlm     = env.register_stellar_asset_contract(admin.clone());

        (env, client, admin, student, xlm)
    }

    fn mint(env: &Env, xlm: &Address, admin: &Address, to: &Address, amount: i128) {
        token::StellarAssetClient::new(env, xlm).mint(to, &amount);
    }

    fn make_hash(env: &Env, val: u8) -> BytesN<32> {
        BytesN::from_array(env, &[val; 32])
    }

    // Test 1 — Happy path: certificate is registered and student receives reward
    #[test]
    fn test_register_and_reward() {
        let (env, client, admin, student, xlm) = setup();

        let hash   = make_hash(&env, 1);
        let reward = 2_000_000_i128;

        // Register certificate
        client.register_certificate(&hash, &student);

        // Verify it exists
        assert!(client.is_verified(&hash));

        // Fund the contract with XLM so it can pay the reward
        mint(&env, &xlm, &admin, &client.address, reward);

        let token_client = token::Client::new(&env, &xlm);
        let bal_before   = token_client.balance(&student);

        // Pay reward
        client.reward_student(&hash, &reward, &xlm);

        let bal_after = token_client.balance(&student);
        assert_eq!(bal_after - bal_before, reward);
    }

    // Test 2 — Edge case: duplicate certificate registration is rejected
    #[test]
    #[should_panic(expected = "certificate already exists")]
    fn test_duplicate_registration_rejected() {
        let (env, client, _admin, student, _xlm) = setup();

        let hash = make_hash(&env, 2);

        client.register_certificate(&hash, &student);
        // Second registration with same hash must panic
        client.register_certificate(&hash, &student);
    }

    // Test 3 — State: storage correctly reflects owner and hash after registration
    #[test]
    fn test_state_after_registration() {
        let (env, client, _admin, student, _xlm) = setup();

        let hash = make_hash(&env, 3);

        client.register_certificate(&hash, &student);

        // Owner must match
        assert_eq!(client.get_owner(&hash), student);

        // Must be verified
        assert!(client.is_verified(&hash));

        // verify_certificate must return true and emit event
        assert!(client.verify_certificate(&hash));
    }
}