#![cfg_attr(not(feature = "std"), no_std)]
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::DispatchResultWithPostInfo;
	use frame_support::pallet_prelude::*;
	use frame_system::ensure_signed;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
	use sp_std::vec::Vec; 

	#[pallet::config] 
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type AssetDepositBase: Get<usize>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
	}

	#[pallet::error] 
	pub enum Error<T> {
		ProofAlreadyClaimed,
		NoSuchProof,
		NotProofOwner,
		ClaimTooLong,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage] 
	#[pallet::getter(fn proofs)]
	pub(super) type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber), ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call] 
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(proof.len() <= T::AssetDepositBase::get(), Error::<T>::ClaimTooLong);
			ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);
			let current_block = <frame_system::Pallet<T>>::block_number();
			Proofs::<T>::insert(&proof, (&sender, current_block));
			Self::deposit_event(Event::ClaimCreated(sender, proof));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn revoke_claim(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);
			let (owner, _) = Proofs::<T>::get(&proof);
			ensure!(sender == owner, Error::<T>::NotProofOwner);
			Proofs::<T>::remove(&proof);
			Self::deposit_event(Event::ClaimRevoked(sender, proof));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer_claim(origin: OriginFor<T>,claim: Vec<u8>,dest: T::AccountId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(claim.len() <= T::AssetDepositBase::get(), Error::<T>::ClaimTooLong);
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::NoSuchProof);
			let (owner, _block_number) = Proofs::<T>::get(&claim);
			ensure!(owner == sender, Error::<T>::NotProofOwner);
			let current_block = <frame_system::Pallet<T>>::block_number();
			Proofs::<T>::insert(&claim, (dest, current_block));
			Ok(().into())
		}
	}
}
