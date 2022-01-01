#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
// 定义pallet模块，并设置pub访问权限
pub mod pallet {
    // 导入模块
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Randomness};
    use frame_system::pallet_prelude::*;
    use codec::{Encode, Decode};
    use sp_io::hashing::blake2_128;
   
    

    	// 定义配置
	#[pallet::config] 
	// 定义trait Config 继承 frame_system::Config trait
	pub trait Config: frame_system::Config {
		// 定义关联类型Event，该类继承 From<Event<Self>> 和 IsType<<Self as frame_system::Config>::Event> 这两个trait
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash,Self::BlockNumber>;
	}

    // 定义事件枚举
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	// 定义泛型枚举，其中枚举类型需继承上面定义的Config trait
	pub enum Event<T: Config> {
        KittyCreated(T::AccountId, KittyIndex),
        KittyTransfer(T::AccountId, T::AccountId, KittyIndex),
    }

    // 定义错误枚举
	#[pallet::error] 
	pub enum Error<T> {
        KittiesCountOverflow,
        AlreadyOwner,
        NotOwner,
        SameParentIndex,
        InvalidKittyIndex
    }


    // 定义Kitty结构体
    #[derive(Encode, Decode, scale_info::TypeInfo)]
    pub struct Kitty(pub [u8;16]);

    // 定义Kitty结构体索引
    type KittyIndex = u32;

    // 定义kitty数量存储结构
    #[pallet::storage]
    #[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T> = StorageValue<_, u32>;

    // 定义kitty集合存储结构
    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<Kitty>, ValueQuery>;

    // 定义所有者存储结构
    #[pallet::storage]
    #[pallet::getter(fn owner)]
    pub type Owner<T:Config> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<T::AccountId>, ValueQuery>;


    // 定义结构体
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    // 定义调用
	#[pallet::call] 
	// 为Pallet结构体扩展方法
	impl<T: Config> Pallet<T> {

        #[pallet::weight(0)]
        pub fn create(origin: OriginFor<T>) -> DispatchResult{
            let who = ensure_signed(origin)?;
            let kitty_id = match Self::kitties_count(){
                Some(id) => {
                    ensure!(id < KittyIndex::max_value(),Error::<T>::KittiesCountOverflow);
                    id + 1
                },
                None =>{
                    1
                }
            };
    
            let  dna = Self::random_value(&who);
            Kitties::<T>::insert(kitty_id,Some(Kitty(dna)));
            Owner::<T>::insert(kitty_id,Some(who.clone()));
            KittiesCount::<T>::put(kitty_id);

            Self::deposit_event(Event::KittyCreated(who,kitty_id));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn transfer(origin: OriginFor<T>, new_owner:T::AccountId, kitty_id:KittyIndex) -> DispatchResult{
            let who = ensure_signed(origin)?;
            ensure!(Some(who.clone())==Owner::<T>::get(kitty_id),Error::<T>::NotOwner);
            Owner::<T>::insert(kitty_id,Some(new_owner.clone()));
            Self::deposit_event(Event::KittyTransfer(who,new_owner,kitty_id));
            Ok(())
        }

      
        #[pallet::weight(0)]
        pub fn breed(origin: OriginFor<T>, kitty_id_1:KittyIndex,kitty_id_2:KittyIndex) -> DispatchResult{
            let who = ensure_signed(origin)?;
            
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);
            
            let kitty1= Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
            let kitty2= Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

            ensure!(Some(who.clone())==Owner::<T>::get(kitty_id_1), Error::<T>::NotOwner);
            ensure!(Some(who.clone())==Owner::<T>::get(kitty_id_2), Error::<T>::NotOwner);

            let kitty_id = match Self::kitties_count(){
                Some(id) => {
                    ensure!(id < KittyIndex::max_value(),Error::<T>::KittiesCountOverflow);
                    id + 1
                },
                None =>{
                    1
                }
            };

            let dna_1 = kitty1.0;
            let dna_2 = kitty2.0;
            let selector = Self::random_value(&who);
            let mut new_dna = [0u8; 16];
            for i in 0..dna_1.len(){
                new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
            }

            Kitties::<T>::insert(kitty_id,Some(Kitty(new_dna)));
            Owner::<T>::insert(kitty_id,Some(who.clone()));
            KittiesCount::<T>::put(kitty_id);

            Self::deposit_event(Event::KittyCreated(who,kitty_id));
           
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn buy(origin: OriginFor<T>, kitty_id:KittyIndex) -> DispatchResult{
            
            // 检查kitty是否存在
            let _kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            
            // 检查kitty是否不是自己所有，防止自己买自己的
            let who = ensure_signed(origin)?;
            let owner = Owner::<T>::get(kitty_id);
            ensure!(Some(who.clone())!= owner, Error::<T>::AlreadyOwner);

            // 修改kitty拥有者
            Owner::<T>::insert(kitty_id, Some(who.clone()));

            Self::deposit_event(Event::KittyTransfer(owner.unwrap(), who, kitty_id));

            Ok(())
        }


        #[pallet::weight(0)]
        pub fn sell(origin: OriginFor<T>, new_owner:T::AccountId, kitty_id:KittyIndex) -> DispatchResult{
            // 检查kitty是否存在
            let _kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

             // 检查是否有kitty所有权
             let who = ensure_signed(origin)?;
             let owner = Owner::<T>::get(kitty_id);
             ensure!(Some(who.clone())== owner, Error::<T>::NotOwner);

             // 修改kitty拥有者
            Owner::<T>::insert(kitty_id, Some(new_owner.clone()));

            Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));

            Ok(())
        }

    }

    impl<T: Config> Pallet<T> {
        fn random_value(sender:&T::AccountId) -> [u8;16]{
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index()
            );
            payload.using_encoded(blake2_128)
        }
    }
}