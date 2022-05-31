mod program_test;

use mango::{
    matching::Side,
    state::{OtcOrderStatus, OtcOrders},
    utils,
};
use program_test::{cookies::*, *};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;

#[tokio::test]
async fn success() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &mango_group_cookie.address;

    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    let (otc_orders_pk, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1000000;
    let size = 2000000000;
    let expires = 9999999999999;

    let counterparty_pk = Pubkey::new_unique();
    test.create_perp_otc_order(
        mango_group_pk,
        &account0_pk,
        &counterparty_pk,
        &mango_group_cookie.perp_markets[0].address,
        0,
        price,
        size,
        expires,
        Side::Ask,
    )
    .await
    .unwrap();

    let otc_orders = test.load_account::<OtcOrders>(otc_orders_pk).await;

    assert_eq!(otc_orders.perp_orders_len, 1);
    assert_eq!(otc_orders.perp_orders[0].status, OtcOrderStatus::Created);
    assert_eq!(otc_orders.perp_orders[0].counterparty_wallet, counterparty_pk);
    assert_eq!(otc_orders.perp_orders[0].creator_side, Side::Ask);
    assert_eq!(otc_orders.perp_orders[0].expires, expires);
    assert_eq!(otc_orders.perp_orders[0].price, price);
    assert_eq!(otc_orders.perp_orders[0].size, size);
    assert_eq!(otc_orders.perp_orders[0].perp_market, mango_group_cookie.perp_markets[0].address);
    assert_eq!(otc_orders.perp_orders[0].perp_account_index, 0);
    assert_eq!(otc_orders.perp_orders[1].status, OtcOrderStatus::Uninitialized);
}

#[tokio::test]
async fn success_canceled_deletion() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &mango_group_cookie.address;

    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    let (otc_orders_pk, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1000000;
    let size = 2000000000;
    let expires = 9999999999999;

    for _ in 0..utils::MAX_PERP_OTC_ORDERS {
        let counterparty_pk = Pubkey::new_unique();
        test.create_perp_otc_order(
            mango_group_pk,
            &account0_pk,
            &counterparty_pk,
            &mango_group_cookie.perp_markets[0].address,
            0,
            price,
            size,
            expires,
            Side::Ask,
        )
        .await
        .unwrap()
    }

    test.cancel_perp_otc_order(mango_group_pk, &account0_pk, 0, 0).await.unwrap();

    let otc_orders = test.load_account::<OtcOrders>(otc_orders_pk).await;

    assert_eq!(otc_orders.perp_orders_len, utils::MAX_PERP_OTC_ORDERS);
    assert_eq!(otc_orders.perp_orders[0].status, OtcOrderStatus::Canceled);

    let counterparty_pk = Pubkey::new_unique();
    test.create_perp_otc_order(
        mango_group_pk,
        &account0_pk,
        &counterparty_pk,
        &mango_group_cookie.perp_markets[0].address,
        0,
        price,
        size,
        expires,
        Side::Ask,
    )
    .await
    .unwrap();

    let otc_orders = test.load_account::<OtcOrders>(otc_orders_pk).await;

    assert_eq!(otc_orders.perp_orders_len, utils::MAX_PERP_OTC_ORDERS);
    assert_eq!(otc_orders.perp_orders[0].status, OtcOrderStatus::Created);
}

#[tokio::test]
async fn fail_invalid_time_argument() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &mango_group_cookie.address;

    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1000000;
    let size = 2000000000;
    let expires = 0;

    let counterparty_pk = Pubkey::new_unique();
    let error = test
        .create_perp_otc_order(
            mango_group_pk,
            &account0_pk,
            &counterparty_pk,
            &mango_group_cookie.perp_markets[0].address,
            0,
            price,
            size,
            expires,
            Side::Ask,
        )
        .await
        .unwrap_err();

    let error_code = get_error_code(error);
    assert_eq!(error_code, Some(41));
}

#[tokio::test]
async fn fail_max_otc_orders_reached() {
    let config = MangoProgramTestConfig::default_two_mints();
    let mut test = MangoProgramTest::start_new(&config).await;

    let mut mango_group_cookie = MangoGroupCookie::default(&mut test).await;
    let num_precreated_mango_users = 0;
    mango_group_cookie
        .full_setup(&mut test, num_precreated_mango_users, config.num_mints - 1)
        .await;

    let mango_group_pk = &mango_group_cookie.address;

    let account0_pk = test.create_mango_account(mango_group_pk, 0, 0, None).await;
    test.create_spot_open_orders(
        mango_group_pk,
        &mango_group_cookie.mango_group,
        &account0_pk,
        0,
        0,
        None,
    )
    .await;

    let (otc_orders_pk, _) = test.init_otc_orders(mango_group_pk, &account0_pk, 0).await;

    let price = 1000000;
    let size = 2000000000;
    let expires = 9999999999999;

    for _ in 0..utils::MAX_PERP_OTC_ORDERS {
        let counterparty_pk = Pubkey::new_unique();
        test.create_perp_otc_order(
            mango_group_pk,
            &account0_pk,
            &counterparty_pk,
            &mango_group_cookie.perp_markets[0].address,
            0,
            price,
            size,
            expires,
            Side::Ask,
        )
        .await
        .unwrap()
    }

    let counterparty_pk = Pubkey::new_unique();
    let error = test
        .create_perp_otc_order(
            mango_group_pk,
            &account0_pk,
            &counterparty_pk,
            &mango_group_cookie.perp_markets[0].address,
            0,
            price,
            size,
            expires,
            Side::Ask,
        )
        .await
        .unwrap_err();

    let otc_orders = test.load_account::<OtcOrders>(otc_orders_pk).await;
    let error_code = get_error_code(error);

    assert_eq!(error_code, Some(42));
    assert_eq!(otc_orders.perp_orders_len, utils::MAX_PERP_OTC_ORDERS);
}
