use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim), (1, frame_system::Pallet::<Test>::block_number()))
	})
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyClaimed,
		);
	});
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
		assert_eq!(Proofs::<Test>::contains_key(&claim), false);
	})
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::NoSuchProof
		);
	});
}

#[test]
fn revoke_claim_failed_when_owner_is_wrong() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let owner_1 = Origin::signed(1);
		let owner_2 = Origin::signed(2);
		let _ = PoeModule::create_claim(owner_1, claim.clone());
		assert_noop!(PoeModule::revoke_claim(owner_2, claim.clone()), Error::<Test>::NotProofOwner);
	})
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let origin = Origin::signed(1);
		let destination = 2;
		let _ = PoeModule::create_claim(origin.clone(), claim.clone());
		assert_ok!(PoeModule::transfer_claim(origin.clone(), claim.clone(), destination));
		assert_eq!(Proofs::<Test>::get(&claim), (2, <frame_system::Pallet<Test>>::block_number()));
		assert_noop!(PoeModule::revoke_claim(origin, claim), Error::<Test>::NotProofOwner);
	});
}

#[test]
fn transfer_claim_failed_when_owner_is_wrong() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let owner_right = Origin::signed(1);
		let owner_wrong = 2;
		let _ = PoeModule::create_claim(owner_right, claim.clone());
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(owner_wrong), claim.clone(), owner_wrong),
			Error::<Test>::NotProofOwner
		);
	});
}

#[test]
fn transfer_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		let claim_red = vec![0, 1];
		let claim_blue = vec![2, 1];
		let origin = Origin::signed(1);
		let destination = 2;
		let _ = PoeModule::create_claim(origin.clone(), claim_red);
		assert_noop!(
			PoeModule::transfer_claim(origin, claim_blue, destination),
			Error::<Test>::NoSuchProof
		);
	})
}

#[test]
fn create_claim_failed_when_claim_too_long() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimTooLong
		);
	})
}
