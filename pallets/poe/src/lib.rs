#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// 配置测试相关模块
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
// 定义pallet模块，并设置pub访问权限
pub mod pallet {
	// 导入模块
	use frame_support::dispatch::DispatchResultWithPostInfo;
	use frame_support::pallet_prelude::*;
	use frame_system::ensure_signed;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
	use sp_std::vec::Vec; 

	// 定义配置
	#[pallet::config] 
	// 定义trait Config 继承 frame_system::Config trait
	pub trait Config: frame_system::Config {
		// 定义关联类型Event，该类继承 From<Event<Self>> 和 IsType<<Self as frame_system::Config>::Event> 这两个trait
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// 定义关联类型AssetDepositBase，该类继承 Get<usize> trait
		type AssetDepositBase: Get<usize>;
	}

	// 定义事件枚举
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	// 定义泛型枚举，其中枚举类型需继承上面定义的Config trait
	pub enum Event<T: Config> {
		//存证创建成功枚举类型,使用元组记录AccountId和相应数据
		ClaimCreated(T::AccountId, Vec<u8>),
		//存证吊销成功枚举类型,使用元组记录AccountId和相应数据
		ClaimRevoked(T::AccountId, Vec<u8>),
	}

	// 定义错误枚举
	#[pallet::error] 
	pub enum Error<T> {
		// 存证已经创建
		ProofAlreadyClaimed,
		// 存证不存在
		NoSuchProof,
		// 非存证所有者
		NotProofOwner,
		// 存证长度超过上限
		ClaimTooLong,
	}

	// 定义结构体
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// 定义存储
	#[pallet::storage] 
	#[pallet::getter(fn proofs)]
	// 对StorageMap结构体重命名，并指定pub(super)访问权限
	pub(super) type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber), ValueQuery>;

	//定义hooks
	#[pallet::hooks]
	// 为Pallet实现Hooks trait
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// 定义调用
	#[pallet::call] 
	// 为Pallet结构体扩展方法
	impl<T: Config> Pallet<T> {

		// 定义权重
		#[pallet::weight(0)]
		// 创建并存储存证
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
		// 销毁存证
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
		// 转移存证
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
