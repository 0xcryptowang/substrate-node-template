#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
	};


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// 定义存储
	
	/// 定义事件
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewNumber(Option<T::AccountId>, u64),
	}

	#[pallet::error]
	pub enum Error<T> {
		UnknownOffchainMux,
		NoLocalAcctForSigning,
		OffchainSignedTxError,
		OffchainUnsignedTxError,
		OffchainUnsignedTxSignedPayloadError,
		HttpFetchingError,
		ConvertError
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		

	}

	impl<T: Config> Pallet<T> {

	}
}
