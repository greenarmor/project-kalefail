use super::*;

#[test]
fn test_constructor() {
    let fixture = TestFixture::create();
    let env = fixture.env;
    let admin = fixture.admin;
    let kale_salad_client = fixture.kale_salad_client;
    let kale_salad_contract = kale_salad_client.address;
    let [(broc, _), (cabb, _), (kohl, _), (brsp, _)] = fixture.vegetables;

    // make sure the storage entries are there
    let stored_metadata: Metadata = env.as_contract(&kale_salad_contract, || {
        env.storage().instance().get(&Storage::Metadata).unwrap()
    });
    assert_eq!(stored_metadata.base_uri, String::from_str(&env, IPFS_URI));
    assert_eq!(stored_metadata.name, String::from_str(&env, NFT_NAME));
    assert_eq!(stored_metadata.symbol, String::from_str(&env, NFT_SYMBOL));

    let stored_admin: Address = env.as_contract(&kale_salad_contract, || {
        env.storage().instance().get(&Storage::Admin).unwrap()
    });
    assert_eq!(admin, stored_admin);

    let stored_vegetables: Vec<Address> = env.as_contract(&kale_salad_contract, || {
        env.storage().instance().get(&Storage::Vegetables).unwrap()
    });
    assert_eq!(
        stored_vegetables,
        vec![&env, broc.address, cabb.address, kohl.address, brsp.address,]
    );

    let payment_per_token: i128 = env.as_contract(&kale_salad_contract, || {
        env.storage()
            .instance()
            .get(&Storage::PaymentPerNft)
            .unwrap()
    });
    assert_eq!(payment_per_token, (10 * 4 * 10_000_000) as i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #104)")]
fn test_cannot_construct_with_too_few_vegetables() {
    let fixture = TestFixture::create();
    let env = fixture.env;
    let [(broc, _), (cabb, _), (kohl, _), _] = fixture.vegetables;

    env.register(
        KaleSaladContract,
        (
            fixture.admin,
            String::from_str(&env, NFT_NAME),
            String::from_str(&env, NFT_SYMBOL),
            String::from_str(&env, IPFS_URI),
            vec![&env, broc.address, cabb.address, kohl.address],
            (10 * 10_000_000) as i128,
        ),
    );
}
