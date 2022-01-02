#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode, EncodeLike};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use num_traits::bounds::Bounded;
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, One};

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub owner: AccountOf<T>,
	}

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type KittyIndex: Parameter + Default + AtLeast32Bit + Copy + Bounded + EncodeLike;

		#[pallet::constant]
		type StakeAmountForKitty: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created(T::AccountId, T::KittyIndex),
		OnSales(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
		Transferred(T::AccountId, T::AccountId, T::KittyIndex),
		Bought(T::AccountId, T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
	}

	/// 定义存储
	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_list_for_sale)]
	pub type ListForSale<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		KittyCntOverflow,
		BuyerIsKittyOwner,
		KittyNotExist,
		NotKittyOwner,
		KittyNotForSale,
		NotEnoughBalance,
		NotEnoughBalanceForStaking,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 创建kitty
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::mint(&who, None)?;
			log::info!("创建了一个kitty,id: {:?}.", kitty_id);
			Self::deposit_event(Event::Created(who, kitty_id));
			Ok(())
		}

		/// 转移kitty
		/// to: 转移到到的账户
		/// kitty_id: 某个具体kitty的索引
		#[pallet::weight(100)]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// 校验是否kitty所有者
			ensure!(Self::is_kitty_owner(&kitty_id, &who)?, <Error<T>>::NotKittyOwner);
			// 转移kitty(内部包含kitty存在性验证)
			Self::transfer_kitty_to(&kitty_id, &to)?;
			log::info!("账户: {:?} 将id为 {:?} 的kitty, 从自己转移到 账户: {:?}.", who, kitty_id, to);
			// 转移成功事件
			Self::deposit_event(Event::Transferred(who, to, kitty_id));
			Ok(())
		}

		/// 孵化kitty
		/// father_kitty_id: 父kitty索引id
		/// mother_kitty_id: 母kitty索引id
		#[pallet::weight(100)]
		pub fn breed(origin: OriginFor<T>, father_kitty_id: T::KittyIndex, mother_kitty_id: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// 生成孵化dna
			let new_dna = Self::breed_dna(&who, father_kitty_id, mother_kitty_id)?;
			// 根据所有者和孵化dna铸造kitty
			let kitty_id = Self::mint(&who, Some(new_dna))?;
			log::info!("账户: {:?} 通过id为 {:?} 的kitty和id为 {:?} 的kitty,孵化出id为 {:?} 的kitty.", who, father_kitty_id, mother_kitty_id, kitty_id);
			// 创建成功事件
			Self::deposit_event(Event::Created(who, kitty_id));
			Ok(())
		}

		
		/// 购买kitty
		/// kitty_id: 某个具体kitty的索引
		#[transactional]
		#[pallet::weight(100)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// 校验是否已经拥有kitty,不能自己买自己的
			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

			// 根据售卖和质押要求校验购买账号
			let ask_price = Self::kitties_list_for_sale(&kitty_id).ok_or(<Error<T>>::KittyNotForSale)?;
			let stake = T::StakeAmountForKitty::get();
			let free_balance = T::Currency::free_balance(&buyer);
			ensure!(free_balance > (ask_price + stake), <Error<T>>::NotEnoughBalance);

			// 转移资产
			let seller = kitty.owner.clone();
			T::Currency::transfer(&buyer, &seller, ask_price, ExistenceRequirement::KeepAlive)?;

			// 转移kitty
			Self::transfer_kitty_to(&kitty_id, &buyer)?;

			// 从售卖列表中移除该kitty
			ListForSale::<T>::remove(kitty_id);
			log::info!("账户: {:?} 花费 {:?} 从 账户: {:?} 购买了id为 {:?} 的kitty.", buyer, ask_price, seller, kitty_id);

			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, Some(ask_price)));

			Ok(())
		}

		/// 上架销售
		/// kitty_id: 某个具体kitty的索引
		/// price； 上架销售价格
		#[pallet::weight(100)]
		pub fn sale(origin: OriginFor<T>, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// kitty必须存在
			ensure!(<Kitties<T>>::contains_key(&kitty_id), <Error<T>>::KittyNotExist);
			// 交易所有者权限
			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, <Error<T>>::NotKittyOwner);

			// 设置价格并上架
			ListForSale::<T>::try_mutate(kitty_id, |p| -> DispatchResult {
				*p = price;
				Ok(().into())
			})?;

			log::info!("账户: {:?} 将id为 {:?} 的kitty上架销售，销售价格为 {:?} .", sender, kitty_id, price);
			Self::deposit_event(Event::OnSales(sender, kitty_id, price));

			Ok(())
		}

	}

	impl<T: Config> Pallet<T> {
		/// 随机值
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		/// kitty所有者身份校验
		/// kitty_id: kitty 索引id
		/// account_id: 账户id
		fn is_kitty_owner(kitty_id: &T::KittyIndex, account_id: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *account_id),
				None => Err(<Error<T>>::KittyNotExist),
			}
		}

		
		/// 铸造
		/// owner: 铸造kitty的所有者
		/// dna: 铸造kitty的dna属性
		pub fn mint(owner: &T::AccountId, dna: Option<[u8; 16]>) -> Result<T::KittyIndex, Error<T>> {
			// 生成dna,并构建kitty
			let dna_inner: [u8; 16];
			if let Some(v) = dna {
				dna_inner = v;
			} else {
				dna_inner = Self::random_value(&owner);
			}
			let kitty = Kitty::<T> { dna: dna_inner, owner: owner.clone() };

			// 构造kitty索引id,第一次索引为1，其余索引为
			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittyCntOverflow);
					id.checked_add(&T::KittyIndex::one()).unwrap()
				}
				None => T::KittyIndex::one()
			};

			// 质押
			let stake = T::StakeAmountForKitty::get();
			T::Currency::reserve(&owner, stake).map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;

			// 存储
			Kitties::<T>::insert(kitty_id, Some(kitty));
			Owner::<T>::insert(kitty_id, Some(owner));
			KittiesCount::<T>::put(kitty_id);

			Ok(kitty_id)
		}

		/// 转移kitty(成功不返回;失败返回错误信息)
		/// kitty_id: kitty索引id
		/// to: 转移kitty到目标账户的accountId
		#[transactional]
		pub fn transfer_kitty_to(kitty_id: &T::KittyIndex,to: &T::AccountId) -> Result<(), Error<T>> {
			// 校验kitty是否存在
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

			// 质押
			let stake = T::StakeAmountForKitty::get();
			T::Currency::reserve(&to, stake).map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;
			T::Currency::unreserve(&kitty.owner, stake);

			// 修改kitty所有者并存储
			kitty.owner = to.clone();
			Kitties::<T>::insert(kitty_id, Some(kitty));
			Owner::<T>::insert(kitty_id, Some(to.clone()));

			Ok(())
		}

		/// 构建dna(成功返回dna;失败返回错误信息)
		/// who: 构建人accountId
		/// father_kitty_id: 父kitty索引id
		/// mother_kitty_id: 母kitty索引id
		fn breed_dna(who: &T::AccountId, father_kitty_id: T::KittyIndex, mother_kitty_id: T::KittyIndex) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::kitties(father_kitty_id).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(mother_kitty_id).ok_or(<Error<T>>::KittyNotExist)?.dna;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna1.len() {
				new_dna[i] = (selector[i] & dna1[i]) | (!selector[i] & dna2[i]);
			}

			Ok(new_dna)
		}
	}
}
