#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use pallet_user::{User, UserId};
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
	pub user_id: UserId,
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
		traits::{Currency, ExistenceRequirement, Get, Randomness, ReservableCurrency},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use scale_info::prelude::vec;
	// use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AccountIdConversion, SaturatedConversion};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
	pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
	pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;

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
		pub user_id: UserId,
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

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_user::Config
		+ orml_nft::Config<TokenData = VineData<AccountOf<Self>>, ClassData = ClassData<BalanceOf<Self>>>
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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn vine_count)]
	pub type VineCount<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vine_storage)]
	pub type VineStorage<T: Config> = StorageValue<_, Vec<UserVines<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn user_vine_storage)]
	pub type VineStorageByUser<T: Config> =
		StorageMap<_, Twox64Concat, UserId, UserVines<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_vines)]
	pub type Vines<T: Config> =
		StorageMap<_, Twox64Concat, VineId, (ClassIdOf<T>, TokenIdOf<T>), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [UserId, VineId, CollectionId, TokenId]
		VineCreated(UserId, VineId, ClassIdOf<T>, TokenIdOf<T>),
		/// [UserId, VineId]
		VineViewed(UserId, VineId),
		/// [Creator, CollectionId, ClassName]
		NewNftCollectionCreated(AccountOf<T>, ClassIdOf<T>, VineMetaData),
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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_collection(
			origin: OriginFor<T>,
			user_id: UserId,
			collection_name: VineMetaData,
			description: VineMetaData,
			thumbnail_image: VineMetaData,
			metadata: VineMetaData,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			let curr_user =
				pallet_user::Users::<T>::get(user_id).ok_or(Error::<T>::UserDoesNotExist)?;

			let next_class_id = <orml_nft::NextClassId<T>>::get();
			log::info!("class_id: {:#?}", next_class_id.clone());

			let nft_account =
				<T as pallet::Config>::PalletId::get().into_sub_account(next_class_id);
			// Secure deposit of token class owner
			let collection_deposit = T::CreateCollectionDeposit::get();
			// Transfer fund to pot
			<T as pallet::Config>::Currency::transfer(
				&creator,
				&nft_account,
				collection_deposit,
				ExistenceRequirement::KeepAlive,
			)?;
			// Reserve pot fund
			<T as pallet::Config>::Currency::reserve(
				&nft_account,
				<T as pallet::Config>::Currency::free_balance(&nft_account),
			)?;

			let new_class_data = ClassData {
				deposit: collection_deposit,
				collection_name: collection_name.clone(),
				description,
				thumbnail_image,
				metadata: metadata.clone(),
			};

			orml_nft::Pallet::<T>::create_class(&creator, metadata, new_class_data)?;

			Self::deposit_event(Event::<T>::NewNftCollectionCreated(
				creator,
				next_class_id,
				collection_name,
			));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_vine(
			origin: OriginFor<T>,
			user_id: UserId,
			collection_id: ClassIdOf<T>,
			vine_description: VineMetaData,
			video_url: VineMetaData,
			thumbnail_image: VineMetaData,
			metadata: VineMetaData,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;

			let curr_user =
				pallet_user::Users::<T>::get(user_id).ok_or(Error::<T>::UserDoesNotExist)?;

			let collection_info = <orml_nft::Classes<T>>::get(collection_id)
				.ok_or(Error::<T>::CollectionDoesNotExist)?;

			ensure!(creator == collection_info.owner, Error::<T>::NoPermission);

			let asset_deposit = T::CreateNftDeposit::get();
			let nft_account =
				<T as pallet::Config>::PalletId::get().into_sub_account(collection_id);

			// Transfer fund to pot
			<T as pallet::Config>::Currency::transfer(
				&creator,
				&nft_account,
				asset_deposit,
				ExistenceRequirement::KeepAlive,
			)?;
			// Reserve pot fund
			<T as pallet::Config>::Currency::reserve(&nft_account, asset_deposit)?;

			let vine_count = Self::increment_vine_counter();

			let new_vine = VineData {
				user_id,
				vine_id: vine_count,
				vine_creator: creator.clone(),
				video_url,
				thumbnail_image,
				vine_description,
				view_count: Default::default(),
				share_count: Default::default(),
				comment_count: Default::default(),
				did_view: Default::default(),
				metadata: metadata.clone(),
			};

			let token_id = orml_nft::Pallet::<T>::mint(
				&creator,
				collection_id,
				metadata.clone(),
				new_vine.clone(),
			)?;
			Vines::<T>::insert(vine_count, (collection_id.clone(), token_id.clone()));

			// if let Some(mut curr_user_vine) = Self::user_vine_storage(user_id) {
			// 	if let Some(ref mut c_vine) = curr_user_vine.created_vines {
			// 		c_vine.push(new_vine.clone());
			// 		VineStorageByUser::<T>::insert(user_id, curr_user_vine);
			// 	}
			// } else {
			// 	let new_user_vines = UserVines::<T> {
			// 		user: curr_user,
			// 		created_vines: Some(vec![new_vine.clone()]),
			// 		watched_vines: None,
			// 	};

			// 	VineStorageByUser::<T>::insert(user_id, new_user_vines);
			// }

			// let updated_user_vine = Self::user_vine_storage(user_id).unwrap();
			// Self::update_all_vine_storage_vec(updated_user_vine);

			Self::deposit_event(Event::<T>::VineCreated(
				user_id,
				vine_count,
				collection_id,
				token_id,
			));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mark_vine_as_viwed(
			origin: OriginFor<T>,
			user_id: UserId,
			vine_id: VineId,
		) -> DispatchResult {
			let _user = ensure_signed(origin)?;

			let _curr_user =
				pallet_user::Users::<T>::get(user_id).ok_or(Error::<T>::UserDoesNotExist)?;

			let mut user_vines =
				Self::user_vine_storage(user_id).ok_or(Error::<T>::UserHasNoVines)?;

			if let Some(ref mut c_vine_vec) = user_vines.created_vines {
				'vine_loop: for vine in c_vine_vec.iter_mut() {
					if vine.vine_id == vine_id {
						vine.did_view = true;
						vine.view_count += 1;
						break 'vine_loop;
					} else {
						Err(Error::<T>::VineDoesNotExist)?;
					}
				}
			}
			VineStorageByUser::<T>::insert(user_id, user_vines);

			let updated_user_vine = Self::user_vine_storage(user_id).unwrap();
			Self::update_all_vine_storage_vec(updated_user_vine);

			Self::deposit_event(Event::<T>::VineViewed(user_id, vine_id));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn calculate_vine_reward(
			origin: OriginFor<T>,
			viewer_id: UserId,
			vine_id: VineId,
			vine_length: u32,
			watched_length: u32,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			let curr_user =
				pallet_user::Users::<T>::get(viewer_id).ok_or(Error::<T>::UserDoesNotExist)?;

			let curr_user_vines = Self::vine_storage().ok_or(Error::<T>::UserHasNoVines)?;

			// check if the  vine_id exists

			let mut curr_user_vine: UserVines<T> = Self::get_user_vine(curr_user_vines, vine_id)?;

			// Creator cannot watch his own video for rewards
			let c_vines = curr_user_vine.created_vines.ok_or(Error::<T>::UserHasNoVines)?;
			ensure!(
				c_vines.iter().find(|v| v.user_id == viewer_id) == None,
				Error::<T>::CreatorCantBeViewer
			);

			let rewards = Self::calculate_viewer_rewards(watched_length, vine_length);

			let new_watched_vine = WatchedVine {
				vine_id,
				vine_length,
				watched_length,
				rewards: Some(rewards.saturated_into()),
				is_watched: true,
			};

			if let Some(mut existing_viewer) = Self::user_vine_storage(viewer_id) {
				if let Some(ref mut w_vine_vec) = existing_viewer.watched_vines {
					// let mut vines = existing_viewer.watched_vines;
					for vine in w_vine_vec.iter_mut() {
						if vine.vine_id != vine_id {
							w_vine_vec.push(new_watched_vine.clone());
							break;
						} else {
							Err(Error::<T>::RewardsAlreadyReceived)?;
						}
					}
					VineStorageByUser::<T>::insert(viewer_id, existing_viewer);
				}
			} else {
				let new_user_watched_data = UserVines::<T> {
					user: curr_user,
					created_vines: None,
					watched_vines: Some(vec![new_watched_vine]),
				};
				VineStorageByUser::<T>::insert(viewer_id, new_user_watched_data);
			}

			// update all_vines storage
			let updated_user_vine = Self::user_vine_storage(viewer_id).unwrap();
			Self::update_all_vine_storage_vec(updated_user_vine);

			Ok(())
		}
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
							return Ok(vine);
						}
					}
				}
			}

			Err(Error::<T>::VineDoesNotExist)
		}

		// fn generate_vine_id() -> [u8; 16] {
		// 	let payload = (Self::vine_count(), <frame_system::Pallet<T>>::block_number());
		// 	payload.using_encode(blake2_128)
		// }
	}
}
