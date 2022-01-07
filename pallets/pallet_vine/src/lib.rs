#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::{
		dispatch::DispatchResult,
		inherent::Vec,
		pallet_prelude::Encode,
		traits::{Currency, ExistenceRequirement, Get, Randomness},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use pallet_user::{User, UserId};
	use scale_info::prelude::vec;
	use scale_info::{prelude::boxed::Box, TypeInfo};
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AccountIdConversion, CheckedAdd, SaturatedConversion};

	type BalanceOf<T> =
		<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;
	pub type VineId = u64;

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct UserVines<T: Config> {
		pub user: User<T>,
		pub vines: Vec<Vine<T>>,
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
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_user::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the vines pallet.
		type Currency: MultiReservableCurrency<Self::AccountId>;

		/// The native currency.
		type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;

		/// The module/pallet identifier.
		type PalletId: Get<PalletId>;
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(u32, T::AccountId),
		/// [UserId, VineId]
		VineCreated(UserId, VineId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		/// Error when the user does not exist
		UserDoesNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_vine(
			origin: OriginFor<T>,
			user_id: UserId,
			vine_description: Vec<u8>,
			video_url: Vec<u8>,
			thumbnail_image: Vec<u8>,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;

			let curr_user =
				pallet_user::Users::<T>::get(user_id).ok_or(Error::<T>::UserDoesNotExist)?;

			let vine_count = Self::increment_vine_counter();

			let new_vine = Vine::<T> {
				user_id,
				vine_id: vine_count,
				vine_creator: creator,
				video_url,
				thumbnail_image,
				vine_description,
				view_count: Default::default(),
				share_count: Default::default(),
				comment_count: Default::default(),
				did_view: Default::default(),
			};

			if let Some(mut curr_user_vine) = Self::user_vine_storage(user_id) {
				curr_user_vine.vines.push(new_vine.clone());
				VineStorageByUser::<T>::insert(user_id, curr_user_vine);
			} else {
				let new_user_vines =
					UserVines::<T> { user: curr_user, vines: vec![new_vine.clone()] };
				VineStorageByUser::<T>::insert(user_id, new_user_vines);
			}

			let updated_user_vine = Self::user_vine_storage(user_id).unwrap();
			Self::update_all_vine_storate_vec(updated_user_vine);

			Self::deposit_event(Event::<T>::VineCreated(user_id, vine_count));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mark_vine_as_viwed(
			origin: OriginFor<T>,
			user_id: UserId,
			vine_id: VineId,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn calculate_vine_reward(origin: OriginFor<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn increment_vine_counter() -> u64 {
			VineCount::<T>::mutate(|n| *n += 1);
			Self::vine_count()
		}

		fn update_all_vine_storate_vec(updated_user_vine: UserVines<T>) {
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

		// fn generate_vine_id() -> [u8; 16] {
		// 	let payload = (Self::vine_count(), <frame_system::Pallet<T>>::block_number());
		// 	payload.using_encode(blake2_128)
		// }
	}
}
