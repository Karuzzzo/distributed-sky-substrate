use crate::mock::*;
use frame_support::{
    assert_noop, assert_ok,
};

// Learn more about testing substrate runtime modules
// https://substrate.dev/docs/en/knowledgebase/runtime/tests
// type Module = super::Module<Test>;
type Timestamp = pallet_timestamp::Module<Test>;
type System = frame_system::Module<Test>;
type Balances = pallet_balances::Module<Test>;
type Error = super::Error<Test>;

#[allow(dead_code)]
// get last events and reset
fn events() -> Vec<TestEvent> {
    let evt = System::events()
        .into_iter()
        .map(|evt| evt.event)
        .collect::<Vec<_>>();
    System::reset_events();
    evt
}
#[allow(dead_code)]
fn last_event() -> TestEvent {
    System::events().pop().expect("Event expected").event
}

#[test]
fn it_default_pallet_transaction_payment_multiplier() {
    use frame_support::sp_runtime::{traits::Convert, FixedPointNumber, FixedU128};

    type Multiplier = FixedU128;
    let multiplier: Multiplier = Multiplier::saturating_from_integer(1);

    let new_multiplier = <() as Convert<Multiplier, Multiplier>>::convert(multiplier);
    println!(
        "default multiplier convert from {:} to {:}",
        multiplier, new_multiplier
    );

    assert_eq!(new_multiplier, 0.into());
}

#[test]
#[cfg(feature = "payment")]
fn it_identity_pallet_transaction_payment_multiplier() {
    use frame_support::sp_runtime::{traits::Convert, FixedPointNumber, FixedU128};

    type Multiplier = FixedU128;
    let multiplier: Multiplier = Multiplier::saturating_from_integer(1);

    let new_multiplier = <crate::payment::IdentityMultiplierUpdater as Convert<
        Multiplier,
        Multiplier,
    >>::convert(multiplier);
    println!(
        "identiry multiplier convert from {:} to {:}",
        multiplier, new_multiplier
    );

    assert_eq!(new_multiplier, multiplier);
}

#[test]
fn it_create_new_account() {
    new_test_ext().execute_with(|| {
        let account = TemplateModule::account_registry(2);
        assert!(!account.is_enable());

        Timestamp::set_timestamp(5000);
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::REGISTRAR_ROLE
        ));

        let account = TemplateModule::account_registry(2);
        assert!(account.is_enable());

        let age = account.age(20000);
        assert_eq!(age, 15000);
    });
}

#[test]
fn it_disable_account() {
    new_test_ext().execute_with(|| {
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(TemplateModule::account_disable(Origin::signed(1), 2));
        assert!(!TemplateModule::account_registry(2).is_enable());
    });
}

#[test]
fn it_try_disable_themself() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TemplateModule::account_disable(Origin::signed(1), 1),
            Error::InvalidAction
        );
        assert!(TemplateModule::account_registry(1).is_enable());
    });
}

#[test]
fn it_try_create_by_registrar() {
    new_test_ext().execute_with(|| {
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::REGISTRAR_ROLE
        ));
        assert_noop!(
            TemplateModule::account_add(Origin::signed(2), 3, super::REGISTRAR_ROLE),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_account_reaped() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::transfer(Origin::signed(1), 2, 10000));
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::REGISTRAR_ROLE
        ));

        assert!(TemplateModule::account_registry(2).is_enable());
        assert_ok!(Balances::transfer(Origin::signed(2), 3, 10000));
        assert!(!TemplateModule::account_registry(2).is_enable());
    });
}

#[test]
fn it_balance() {
    new_test_ext().execute_with(|| {
        //total_issuance
        assert_eq!(Balances::total_issuance(), 100000);
        assert_eq!(Balances::free_balance(1), 100000);
        assert_eq!(Balances::free_balance(2), 0);
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            3,
            super::REGISTRAR_ROLE
        ));

        assert_ok!(Balances::transfer(Origin::signed(1), 2, 50000));

        assert_eq!(Balances::free_balance(1), 50000);
        assert_eq!(Balances::free_balance(2), 50000);
        assert_eq!(Balances::total_issuance(), 100000);
        assert_ok!(Balances::transfer(Origin::signed(1), 2, 49990));
        // Account 1 has been removed from balances and dust remaining 10
        assert_eq!(Balances::total_issuance(), 99990);
    });
}

