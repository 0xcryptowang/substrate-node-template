#![cfg_attr(not(feature = "std"), no_std)]
include!("./github.rs");
include!("./dot.rs");

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	/// 声明引用
	use core::{convert::TryInto};
	use parity_scale_codec::{Decode, Encode};
	use frame_support::pallet_prelude::*;
	use frame_system::{
		pallet_prelude::*,
		offchain::{
			AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
			SignedPayload, Signer, SigningTypes, SubmitTransaction,
		},
	};
	use sp_core::{crypto::KeyTypeId};
	use sp_arithmetic::per_things::Permill;
	use sp_runtime::{
		offchain as rt_offchain,
		traits::{
			BlockNumberProvider
		},
		offchain::{
			storage::StorageValueRef,
			storage_lock::{BlockAndTime, StorageLock},
		},
		transaction_validity::{
			InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
		},
		RuntimeDebug,
	};
	use sp_std::{collections::vec_deque::VecDeque, prelude::*, str};
	use serde::{Deserialize};
	

	/// 定义常量
	pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");
	const NUM_VEC_LEN: usize = 10;
	const UNSIGNED_TXS_PRIORITY: u64 = 100;
	const LOCK_BLOCK_EXPIRATION: u32 = 3;
	const LOCK_TIMEOUT_EXPIRATION: u64 = 4000;
	
	pub mod crypto {
		use crate::KEY_TYPE;
		use sp_core::sr25519::Signature as Sr25519Signature;
		use sp_runtime::app_crypto::{app_crypto, sr25519};
		use sp_runtime::{traits::Verify, MultiSignature, MultiSigner};

		app_crypto!(sr25519, KEY_TYPE);

		pub struct TestAuthId;

		impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
			type RuntimeAppPublic = Public;
			type GenericSignature = sp_core::sr25519::Signature;
			type GenericPublic = sp_core::sr25519::Public;
		}

		impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
			type RuntimeAppPublic = Public;
			type GenericSignature = sp_core::sr25519::Signature;
			type GenericPublic = sp_core::sr25519::Public;
		}
	}


	/// 定义结构体
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
	pub struct Payload<Public> {
		number: u64,
		public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}


	#[derive(Debug, Deserialize, Encode, Decode, Default)]
	struct IndexingData(Vec<u8>, u64);

	
	/// 定义pallet
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);	


	/// 定义config
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: From<Call<Self>>;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}


    /// 定义存储
	#[pallet::storage]
	#[pallet::getter(fn numbers)]
	pub type Numbers<T> = StorageValue<_, VecDeque<u64>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn prices)]
	pub type Prices<T> = StorageValue<_, VecDeque<(u64, Permill)>, ValueQuery>;


	/// 定义事件
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewNumber(Option<T::AccountId>, u64),
	}

	/// 定义error
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

	/// 定义 pallet hooks
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		
		
	}

	/// 定义pallet call
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
	}
}
