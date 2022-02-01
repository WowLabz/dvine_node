#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{inherent::Vec, pallet_prelude::*};
use frame_system::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use pallet_user::{User, UserMetaData};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type VineMetaData = Vec<u8>;
pub type VineId = u64;

#[derive(Encode, Decode, Clone, TypeInfo, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData<Balance> {
	pub deposit: Balance,
	pub collection_name: VineMetaData,
	pub description: VineMetaData,
	pub thumbnail_image: VineMetaData,
	pub metadata: VineMetaData,
}

#[derive(Encode, Decode, Clone, TypeInfo, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VineData<AccountId> {
	pub email: UserMetaData,
	pub vine_id: VineId,
	pub vine_creator: AccountId,
	pub video_url: VineMetaData,
	pub thumbnail_image: VineMetaData,
	pub vine_description: VineMetaData,
	pub view_count: u64,
	pub share_count: u64,
	pub comment_count: u64,
	pub did_view: bool,
	pub metadata: VineMetaData,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		inherent::Vec,
		traits::{
			Currency, EnsureOrigin, ExistenceRequirement, Get, Randomness, ReservableCurrency,
		},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use scale_info::prelude::vec;
	// use sp_io::hashing::blake2_128;
	use sp_runtime::{
		traits::{AccountIdConversion, SaturatedConversion, Verify},
		AccountId32, MultiSignature,
	};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
	pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
	pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;

	pub const VINE_CREATED_REWARD_AMT: u128 = 10000000000000;
	pub const VINE_VIEWED_REWARD_AMT: u128 = 5000000000000;

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct UserVines<T: Config> {
		pub user: User<T>,
		pub created_vines: Option<Vec<Vine<T>>>,
		pub watched_vines: Option<Vec<WatchedVine<T>>>,
	}

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct Vine<T: Config> {
		pub email: UserMetaData,
		pub vine_id: VineId,
		pub vine_creator: AccountOf<T>,
		pub video_url: Vec<u8>,
		pub thumbnail_image: Vec<u8>,
		pub vine_description: Vec<u8>,
		pub view_count: u64,
		pub share_count: u64,
		pub comment_count: u64,
		pub did_view: bool,
		pub metadata: VineMetaData,
	}

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct WatchedVine<T: Config> {
		vine_id: VineId,
		vine_length: u32,
		watched_length: u32,
		rewards: Option<BalanceOf<T>>,
		is_watched: bool,
	}

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct RewardsInfo<T: Config> {
		created_vine_rewards: Option<BalanceOf<T>>,
		viewed_vine_rewards: Option<BalanceOf<T>>,
	}

	impl<T: Config> RewardsInfo<T> {
		fn new() -> Self {
			Self { created_vine_rewards: None, viewed_vine_rewards: None }
		}

		fn set_created_vine_rewards(&mut self, amt: BalanceOf<T>) {
			if let Some(mut reward) = self.created_vine_rewards {
				log::info!("1");
				self.created_vine_rewards = Some(reward + amt);
			} else {
				log::info!("2");
				self.created_vine_rewards = Some(amt);
			}
		}

		fn set_viewed_vine_rewards(&mut self, amt: BalanceOf<T>) {
			if let Some(mut reward) = self.viewed_vine_rewards {
				log::info!("3");
				self.viewed_vine_rewards = Some(reward + amt);
			} else {
				log::info!("4");
				self.viewed_vine_rewards = Some(amt);
			}
		}
	}

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	pub enum RewardType {
		CreatorReward,
		ViewerReward,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_user::Config
		+ orml_nft::Config<
			TokenData = VineData<AccountOf<Self>>,
			ClassData = ClassData<BalanceOf<Self>>,
		>
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Currency type for reserve/unreserve balance
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The module/pallet identifier.
		type PalletId: Get<PalletId>;

		/// The minimum balance to create class
		#[pallet::constant]
		type CreateCollectionDeposit: Get<BalanceOf<Self>>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateNftDeposit: Get<BalanceOf<Self>>;

		// DummyAccountWithBalanceForTest Account Id
		type DummyAccountWithBalanceForTest: Get<<Self as frame_system::Config>::AccountId>;
		// type DummyAccountWithBalanceForTest: Get<AccountOf<Self>>;
		// type DummyAccountWithBalanceForTest: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// counter for pallet_vine
	#[pallet::storage]
	#[pallet::getter(fn vine_count)]
	pub type VineCount<T> = StorageValue<_, u64, ValueQuery>;

	// not used in phase1
	#[pallet::storage]
	#[pallet::getter(fn vine_storage)]
	pub type VineStorage<T: Config> = StorageValue<_, Vec<UserVines<T>>, OptionQuery>;

	// not used in phase1
	#[pallet::storage]
	#[pallet::getter(fn user_vine_storage)]
	pub type VineStorageByUser<T: Config> =
		StorageMap<_, Twox64Concat, UserMetaData, UserVines<T>, OptionQuery>;

	// storage with vine_id and (collection_id, vine_id)
	#[pallet::storage]
	#[pallet::getter(fn get_vines)]
	pub type Vines<T: Config> =
		StorageMap<_, Twox64Concat, VineId, (ClassIdOf<T>, TokenIdOf<T>), OptionQuery>;

	// storage to get the collection id for a user
	#[pallet::storage]
	#[pallet::getter(fn get_collection_id_by_user)]
	pub type CollectionIdByUser<T: Config> =
		StorageMap<_, Twox64Concat, UserMetaData, ClassIdOf<T>, OptionQuery>;

	// storage for all the vines
	#[pallet::storage]
	#[pallet::getter(fn all_vines)]
	pub type AllVines<T: Config> =
		StorageMap<_, Twox64Concat, u32, Vec<VineData<AccountOf<T>>>, OptionQuery>;

	// storage for user reward info
	#[pallet::storage]
	#[pallet::getter(fn get_user_rewards_info)]
	pub type UserRewards<T: Config> =
		StorageMap<_, Twox64Concat, UserMetaData, RewardsInfo<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [EmailId, CollectionId, VineId]
		VineCreated(UserMetaData, ClassIdOf<T>, TokenIdOf<T>),
		/// [EmailId, CollectionId, VineId]
		VineViewed(UserMetaData, ClassIdOf<T>, TokenIdOf<T>),
		/// [Creator, CollectionId, CollectionName]
		NewNftCollectionCreated(AccountOf<T>, ClassIdOf<T>, VineMetaData),
		/// [VineCount]
		AllVinesStorageUpdated(u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error when the user does not exist
		UserDoesNotExist,
		/// Error when user has not vines
		UserHasNoVines,
		/// Error when vine by the given vine_id does not exist
		VineDoesNotExist,
		/// Error when trying to get rewards for an already viewed vine
		RewardsAlreadyReceived,
		/// Error when the creator is calculate rewards for his own viewing
		CreatorCantBeViewer,
		/// Class of nft already exists
		CollectionAlreadyExists,
		/// Deposit for class or nft is below minimum
		DepositTooLow,
		/// Collection does not exist
		CollectionDoesNotExist,
		/// Not authorized
		NoPermission,
		/// Creator cannot be the viewer to get rewards
		CreatorCannotBeTheViewer,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn instant_vine_creation(
			origin: OriginFor<T>,
			email: UserMetaData,
			vine_description: VineMetaData,
			video_url: VineMetaData,
			thumbnail_image: VineMetaData,
			metadata: Option<VineMetaData>,
		) -> DispatchResult {
			let creator = ensure_signed(origin.clone())?;

			let curr_user =
				pallet_user::Users::<T>::get(email.clone()).ok_or(Error::<T>::UserDoesNotExist)?;

			// creating a default class for the user
			let curr_collection_id =
				Self::check_and_create_user_collection_id(email.clone(), creator.clone())?;
			log::info!("instant_create_vine class_id: {:#?}", curr_collection_id.clone());

			let nft_account: AccountOf<T> = <T as pallet::Config>::PalletId::get().into_account();
			log::info!("instant_create_vine pallet_id_acc: {:#?}", nft_account.clone());
			let nft_sub_acc: AccountOf<T> =
				<T as pallet::Config>::PalletId::get().into_sub_account(curr_collection_id.clone());
			log::info!("instant_create_vine pallet_id_sub_acc: {:#?}", nft_sub_acc);

			// Secure deposit of the collection and token class owner
			let collection_deposit = T::CreateCollectionDeposit::get();
			let asset_deposit = T::CreateNftDeposit::get();
			let total_deposit = collection_deposit + asset_deposit;

			type Signature = MultiSignature;
			type AccountPublic = <Signature as Verify>::Signer;

			let alice_acc = <T as pallet::Config>::DummyAccountWithBalanceForTest::get();
			log::info!("alice_acc from runtime: {:#?}", alice_acc.clone());

			// Transfer fund to pot
			<T as pallet::Config>::Currency::transfer(
				&alice_acc,
				&nft_account,
				total_deposit,
				ExistenceRequirement::KeepAlive,
			)?;

			// Reserve pot fund
			// <T as pallet::Config>::Currency::reserve(
			// 	&nft_account,
			// 	<T as pallet::Config>::Currency::free_balance(&nft_account),
			// )?;

			let collection_info = <orml_nft::Classes<T>>::get(curr_collection_id.clone())
				.ok_or(Error::<T>::CollectionDoesNotExist)?;

			ensure!(creator.clone() == collection_info.owner, Error::<T>::NoPermission);

			let vine_count = Self::increment_vine_counter();
			// let vine_count = orml_nft::NextTokenId::<T>::get(curr_collection_id.clone());

			let new_vine = VineData {
				email: email.clone(),
				vine_id: vine_count,
				vine_creator: creator.clone(),
				video_url,
				thumbnail_image,
				vine_description,
				view_count: Default::default(),
				share_count: Default::default(),
				comment_count: Default::default(),
				did_view: Default::default(),
				metadata: metadata.clone().unwrap_or(b"metadata".to_vec()),
			};

			let token_id = orml_nft::Pallet::<T>::mint(
				&creator,
				curr_collection_id.clone(),
				metadata.clone().unwrap_or(b"metadata".to_vec()),
				new_vine.clone(),
			)?;

			Vines::<T>::insert(vine_count.clone(), (curr_collection_id.clone(), token_id.clone()));

			// at vine creation the creator is rewarded
			// with 1 native token as reward
			<T as pallet::Config>::Currency::transfer(
				&nft_account,
				&creator,
				VINE_CREATED_REWARD_AMT.clone().saturated_into(),
				ExistenceRequirement::KeepAlive,
			)?;

			Self::update_rewards_for_user(
				email.clone(),
				VINE_CREATED_REWARD_AMT.clone().saturated_into(),
				RewardType::CreatorReward,
			);

			Self::deposit_event(Event::<T>::VineCreated(
				email,
				curr_collection_id.clone(),
				token_id,
			));

			Ok(())
		}

		// Commented for use in Phase2
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn create_collection(
		// 	origin: OriginFor<T>,
		// 	user_id: UserId,
		// 	collection_name: VineMetaData,
		// 	description: VineMetaData,
		// 	thumbnail_image: VineMetaData,
		// 	metadata: VineMetaData,
		// ) -> DispatchResultWithPostInfo {
		// 	let creator = ensure_signed(origin)?;

		// 	let curr_user =
		// 		pallet_user::Users::<T>::get(user_id).ok_or(Error::<T>::UserDoesNotExist)?;

		// 	let next_class_id = <orml_nft::NextClassId<T>>::get();
		// 	log::info!("class_id: {:#?}", next_class_id.clone());

		// 	let nft_account =
		// 		<T as pallet::Config>::PalletId::get().into_sub_account(next_class_id);
		// 	// Secure deposit of token class owner
		// 	let collection_deposit = T::CreateCollectionDeposit::get();
		// 	// Transfer fund to pot
		// 	<T as pallet::Config>::Currency::transfer(
		// 		&creator,
		// 		&nft_account,
		// 		collection_deposit,
		// 		ExistenceRequirement::KeepAlive,
		// 	)?;
		// 	// Reserve pot fund
		// 	<T as pallet::Config>::Currency::reserve(
		// 		&nft_account,
		// 		<T as pallet::Config>::Currency::free_balance(&nft_account),
		// 	)?;

		// 	let new_class_data = ClassData {
		// 		deposit: collection_deposit,
		// 		collection_name: collection_name.clone(),
		// 		description,
		// 		thumbnail_image,
		// 		metadata: metadata.clone(),
		// 	};

		// 	orml_nft::Pallet::<T>::create_class(&creator, metadata, new_class_data)?;

		// 	Self::deposit_event(Event::<T>::NewNftCollectionCreated(
		// 		creator,
		// 		next_class_id,
		// 		collection_name,
		// 	));

		// 	Ok(().into())
		// }

		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn create_vine(
		// 	origin: OriginFor<T>,
		// 	user_id: UserId,
		// 	collection_id: ClassIdOf<T>,
		// 	vine_description: VineMetaData,
		// 	video_url: VineMetaData,
		// 	thumbnail_image: VineMetaData,
		// 	metadata: VineMetaData,
		// ) -> DispatchResult {
		// 	let creator = ensure_signed(origin)?;

		// 	let curr_user =
		// 		pallet_user::Users::<T>::get(user_id).ok_or(Error::<T>::UserDoesNotExist)?;

		// 	let collection_info = <orml_nft::Classes<T>>::get(collection_id)
		// 		.ok_or(Error::<T>::CollectionDoesNotExist)?;

		// 	ensure!(creator == collection_info.owner, Error::<T>::NoPermission);

		// 	let asset_deposit = T::CreateNftDeposit::get();
		// 	let nft_account =
		// 		<T as pallet::Config>::PalletId::get().into_sub_account(collection_id);

		// 	// Transfer fund to pot
		// 	<T as pallet::Config>::Currency::transfer(
		// 		&creator,
		// 		&nft_account,
		// 		asset_deposit,
		// 		ExistenceRequirement::KeepAlive,
		// 	)?;
		// 	// Reserve pot fund
		// 	<T as pallet::Config>::Currency::reserve(&nft_account, asset_deposit)?;

		// 	let vine_count = Self::increment_vine_counter();

		// 	let new_vine = VineData {
		// 		user_id,
		// 		vine_id: vine_count,
		// 		vine_creator: creator.clone(),
		// 		video_url,
		// 		thumbnail_image,
		// 		vine_description,
		// 		view_count: Default::default(),
		// 		share_count: Default::default(),
		// 		comment_count: Default::default(),
		// 		did_view: Default::default(),
		// 		metadata: metadata.clone(),
		// 	};

		// 	let token_id = orml_nft::Pallet::<T>::mint(
		// 		&creator,
		// 		collection_id,
		// 		metadata.clone(),
		// 		new_vine.clone(),
		// 	)?;
		// 	Vines::<T>::insert(vine_count, (collection_id.clone(), token_id.clone()));

		// 	// if let Some(mut curr_user_vine) = Self::user_vine_storage(user_id) {
		// 	// 	if let Some(ref mut c_vine) = curr_user_vine.created_vines {
		// 	// 		c_vine.push(new_vine.clone());
		// 	// 		VineStorageByUser::<T>::insert(user_id, curr_user_vine);
		// 	// 	}
		// 	// } else {
		// 	// 	let new_user_vines = UserVines::<T> {
		// 	// 		user: curr_user,
		// 	// 		created_vines: Some(vec![new_vine.clone()]),
		// 	// 		watched_vines: None,
		// 	// 	};

		// 	// 	VineStorageByUser::<T>::insert(user_id, new_user_vines);
		// 	// }

		// 	// let updated_user_vine = Self::user_vine_storage(user_id).unwrap();
		// 	// Self::update_all_vine_storage_vec(updated_user_vine);

		// 	Self::deposit_event(Event::<T>::VineCreated(
		// 		user_id,
		// 		vine_count,
		// 		collection_id,
		// 		token_id,
		// 	));
		// 	Ok(())
		// }

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mark_vine_as_viwed(
			origin: OriginFor<T>,
			viewer_email: UserMetaData,
			vine_id: TokenIdOf<T>,
		) -> DispatchResult {
			let viewer = ensure_signed(origin)?;

			let _curr_user =
				pallet_user::Users::<T>::get(viewer_email.clone()).ok_or(Error::<T>::UserDoesNotExist)?;

			let (vine_collection_id, vine_token_id) =
				Self::get_vines(vine_id).ok_or(Error::<T>::VineDoesNotExist)?;

			let mut token_info =
				<orml_nft::Tokens<T>>::get(vine_collection_id.clone(), vine_token_id.clone())
					.ok_or(Error::<T>::VineDoesNotExist)?;

			ensure!(viewer_email.clone() != token_info.data.email, Error::<T>::CreatorCannotBeTheViewer);

			token_info.data.view_count += 1;
			token_info.data.did_view = true;

			<orml_nft::Tokens<T>>::insert(
				vine_collection_id.clone(),
				vine_token_id.clone(),
				token_info.clone(),
			);

			let nft_account: AccountOf<T> = <T as pallet::Config>::PalletId::get().into_account();

			// viwere is rewarded
			<T as pallet::Config>::Currency::transfer(
				&nft_account,
				&viewer,
				VINE_VIEWED_REWARD_AMT.saturated_into(),
				ExistenceRequirement::KeepAlive,
			)?;

			Self::update_rewards_for_user(
				viewer_email.clone(),
				VINE_VIEWED_REWARD_AMT.clone().saturated_into(),
				RewardType::ViewerReward,
			);

			// creator is rewarded
			<T as pallet::Config>::Currency::transfer(
				&nft_account,
				&token_info.owner,
				VINE_VIEWED_REWARD_AMT.saturated_into(),
				ExistenceRequirement::KeepAlive,
			)?;

			Self::update_rewards_for_user(
				token_info.data.email.clone(),
				VINE_VIEWED_REWARD_AMT.clone().saturated_into(),
				RewardType::CreatorReward,
			);

			Self::deposit_event(Event::<T>::VineViewed(
				viewer_email,
				vine_collection_id,
				vine_token_id,
			));

			// Commented for use in Phase2
			// if let Some(ref mut c_vine_vec) = user_vines.created_vines {
			// 	'vine_loop: for vine in c_vine_vec.iter_mut() {
			// 		if vine.vine_id == vine_id {
			// 			vine.did_view = true;
			// 			vine.view_count += 1;
			// 			break 'vine_loop;
			// 		} else {
			// 			Err(Error::<T>::VineDoesNotExist)?;
			// 		}
			// 	}
			// }
			// VineStorageByUser::<T>::insert(user_id, user_vines);

			// let updated_user_vine = Self::user_vine_storage(user_id).unwrap();
			// Self::update_all_vine_storage_vec(updated_user_vine);

			// Self::deposit_event(Event::<T>::VineViewed(user_id, vine_id));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn get_all_vines(origin: OriginFor<T>) -> DispatchResult {
			let client = ensure_signed(origin)?;

			let mut all_vines: Vec<VineData<AccountOf<T>>> = Vec::new();

			for (coll_id, token_id, token_info) in <orml_nft::Tokens<T>>::iter() {
				all_vines.push(token_info.data);
			}

			log::info!("all_vines_vec: {:#?}", all_vines);

			AllVines::<T>::insert(1u32, all_vines.clone());

			Self::deposit_event(Event::<T>::AllVinesStorageUpdated(all_vines.len() as u32));

			Ok(())
		}

		// Commented for use in Phase2
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn calculate_vine_reward(
		// 	origin: OriginFor<T>,
		// 	viewer_id: UserId,
		// 	vine_id: VineId,
		// 	vine_length: u32,
		// 	watched_length: u32,
		// ) -> DispatchResult {
		// 	let user = ensure_signed(origin)?;

		// 	let curr_user =
		// 		pallet_user::Users::<T>::get(viewer_id).ok_or(Error::<T>::UserDoesNotExist)?;

		// 	let curr_user_vines = Self::vine_storage().ok_or(Error::<T>::UserHasNoVines)?;

		// 	// check if the  vine_id exists

		// 	let mut curr_user_vine: UserVines<T> = Self::get_user_vine(curr_user_vines, vine_id)?;

		// 	// Creator cannot watch his own video for rewards
		// 	let c_vines = curr_user_vine.created_vines.ok_or(Error::<T>::UserHasNoVines)?;
		// 	ensure!(
		// 		c_vines.iter().find(|v| v.user_id == viewer_id) == None,
		// 		Error::<T>::CreatorCantBeViewer
		// 	);

		// 	let rewards = Self::calculate_viewer_rewards(watched_length, vine_length);

		// 	let new_watched_vine = WatchedVine {
		// 		vine_id,
		// 		vine_length,
		// 		watched_length,
		// 		rewards: Some(rewards.saturated_into()),
		// 		is_watched: true,
		// 	};

		// 	if let Some(mut existing_viewer) = Self::user_vine_storage(viewer_id) {
		// 		if let Some(ref mut w_vine_vec) = existing_viewer.watched_vines {
		// 			// let mut vines = existing_viewer.watched_vines;
		// 			for vine in w_vine_vec.iter_mut() {
		// 				if vine.vine_id != vine_id {
		// 					w_vine_vec.push(new_watched_vine.clone());
		// 					break;
		// 				} else {
		// 					Err(Error::<T>::RewardsAlreadyReceived)?;
		// 				}
		// 			}
		// 			VineStorageByUser::<T>::insert(viewer_id, existing_viewer);
		// 		}
		// 	} else {
		// 		let new_user_watched_data = UserVines::<T> {
		// 			user: curr_user,
		// 			created_vines: None,
		// 			watched_vines: Some(vec![new_watched_vine]),
		// 		};
		// 		VineStorageByUser::<T>::insert(viewer_id, new_user_watched_data);
		// 	}

		// 	// update all_vines storage
		// 	let updated_user_vine = Self::user_vine_storage(viewer_id).unwrap();
		// 	Self::update_all_vine_storage_vec(updated_user_vine);

		// 	Ok(())
		// }
	}

	impl<T: Config> Pallet<T> {
		fn increment_vine_counter() -> u64 {
			VineCount::<T>::mutate(|n| *n += 1);
			Self::vine_count()
		}

		fn update_all_vine_storage_vec(updated_user_vine: UserVines<T>) {
			if let Some(mut all_vines) = Self::vine_storage() {
				if let Some(vine_index) =
					all_vines.iter().position(|vine| vine.user.id == updated_user_vine.user.id)
				{
					all_vines[vine_index] = updated_user_vine.clone();
				} else {
					all_vines.push(updated_user_vine);
				}
				VineStorage::<T>::put(all_vines);
			} else {
				VineStorage::<T>::put(vec![updated_user_vine.clone()]);
			}
		}

		fn calculate_viewer_rewards(watched_length: u32, vine_length: u32) -> u128 {
			if watched_length == vine_length {
				1
			} else {
				0
			}
		}

		fn calculate_creator_rewards(created_vines: Vec<Vine<T>>) -> u128 {
			let mut rewards = 0;

			for vine in created_vines {
				rewards += vine.view_count;
			}

			rewards.into()
		}

		fn get_user_vine(
			all_vines: Vec<UserVines<T>>,
			vine_id: VineId,
		) -> Result<UserVines<T>, Error<T>> {
			for vine in all_vines.into_iter() {
				if let Some(ref c_vine_vec) = vine.created_vines {
					for c_vine in c_vine_vec {
						if c_vine.vine_id == vine_id {
							return Ok(vine)
						}
					}
				}
			}

			Err(Error::<T>::VineDoesNotExist)
		}

		fn check_and_create_user_collection_id(
			email: UserMetaData,
			creator: AccountOf<T>,
		) -> Result<ClassIdOf<T>, DispatchError> {
			if let Some(existing_class_id) = Self::get_collection_id_by_user(email.clone()) {
				Ok(existing_class_id)
			} else {
				let default_class = b"default".to_vec();

				let collection_deposit = T::CreateCollectionDeposit::get();

				let new_class_data = ClassData {
					deposit: collection_deposit,
					collection_name: default_class.clone(),
					description: default_class.clone(),
					thumbnail_image: default_class.clone(),
					metadata: default_class.clone(),
				};

				let next_class_id = <orml_nft::NextClassId<T>>::get();

				let created_class_id =
					orml_nft::Pallet::<T>::create_class(&creator, default_class, new_class_data)?;

				CollectionIdByUser::<T>::insert(email, created_class_id);

				Ok(created_class_id)
			}
		}

		fn update_rewards_for_user(
			email: UserMetaData,
			reward_amt: BalanceOf<T>,
			reward_type: RewardType,
		) {
			// let mut new_reward_info = RewardsInfo::new();

			let mut update_reward_info: RewardsInfo<T> =
				Self::get_user_rewards_info(email.clone()).unwrap_or(RewardsInfo::new());

			// let mut update_reward_info: RewardsInfo<T> =
			// 	Self::get_user_rewards_info(user_id.clone()).unwrap();
			log::info!(
				"update_reward_info create: {:#?}",
				update_reward_info.clone().created_vine_rewards
			);
			log::info!(
				"update_reward_info view: {:#?}",
				update_reward_info.clone().viewed_vine_rewards
			);
			log::info!("reward_type: {:#?}", reward_type);

			match reward_type {
				RewardType::CreatorReward => {
					log::info!("5");
					update_reward_info.set_created_vine_rewards(reward_amt.clone())
				},
				RewardType::ViewerReward => {
					log::info!("6");
					update_reward_info.set_viewed_vine_rewards(reward_amt.clone())
				},
			};

			UserRewards::<T>::insert(email.clone(), update_reward_info);
		}

		// fn generate_vine_id() -> [u8; 16] {
		// 	let payload = (Self::vine_count(), <frame_system::Pallet<T>>::block_number());
		// 	payload.using_encode(blake2_128)
		// }
	}
}
