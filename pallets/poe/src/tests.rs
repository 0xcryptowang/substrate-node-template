use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_claim_works(){
     new_test_ext().execute_with(||{
         let claim = vec![1,2];
        
         assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
         assert_eq!(
              Proofs::<Test>::get(&claim),
              Some((1, frame_system::Pallet::<Test>::block_number()))
            );
        
     })
}

#[test]
fn create_claim_failed_when_claim_already_exist(){
    new_test_ext().execute_with(||{
        let claim = vec![1,2];
        let _= PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_noop!(PoeModule::create_claim(Origin::signed(1), claim.clone()),Error::<Test>::ProofAlreadyExist);
    })
}




