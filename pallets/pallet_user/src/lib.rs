#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod curves;

#[frame_support::pallet]
pub mod pallet {
	use crate::curves::*;
	use frame_support::{
		dispatch::DispatchResult,
		inherent::Vec,
		pallet_prelude::{OptionQuery, *},
		traits::{Currency, ExistenceRequirement, Get, Randomness},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use scale_info::{prelude::boxed::Box, TypeInfo};
	use sp_runtime::traits::{AccountIdConversion, CheckedAdd, SaturatedConversion};

	type BalanceOf<T> =
		<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;
	type UserId = u64;

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct User<T: Config> {
		pub(super) id: UserId,
		pub(super) name: Vec<u8>,
		pub(super) profile_image: Vec<u8>,
		pub(super) vines_count: Option<u64>,
		pub(super) is_following: bool,
		pub(super) accounts: Vec<AccountOf<T>>,
		pub(super) token_info: TokenInfo<T>,
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct TokenInfo<T: Config> {
		pub(super) token_id: CurrencyIdOf<T>,
		pub(super) curve_id: u64,
		pub(super) creator: AccountOf<T>,
		pub(super) curve_type: CurveType,
		pub(super) token_name: Vec<u8>,
		pub(super) token_symbol: Vec<u8>,
		pub(super) token_decimals: u8,
		pub(super) max_supply: BalanceOf<T>,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: MultiReservableCurrency<Self::AccountId>;

		/// The native currency.
		type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;

		/// The deposit required for creating a new bonding curve.
		type CurveDeposit: Get<BalanceOf<Self>>;

		/// The deposit required for creating a new asset with bonding curve.
		type CreatorAssetDeposit: Get<BalanceOf<Self>>;

		/// The module/pallet identifier.
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_id)]
	pub(super) type NextId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn users)]
	pub(super) type Users<T: Config> = StorageMap<_, Twox64Concat, UserId, User<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tokens_storage)]
	pub(super) type TokensStorage<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, User<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [UserId, UserName, TokenId, TokenName]
		UserWithTokenCreated(UserId, Vec<u8>, CurrencyIdOf<T>, Vec<u8>),
		/// (Minter, TokenId, MintAmount, Cost)
		TokenMint(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Sender does not have enough base currency to reserve for a new curve.
		InsufficientBalanceToReserve,
		/// The token that is trying to be created already exists.
		TokenAlreadyExists,
		/// Error when an creator token does not exist
		TokenDoesNotExist,
		/// Error when the token max supply exceeds
		TokenMaxSupplyExceeded,
		/// Sender does not have enough base currency to make a purchase.
		InsufficentBalanceForPurchase,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_user_with_token(
			origin: OriginFor<T>,
			user_name: Vec<u8>,
			profile_image: Vec<u8>,
			vines_count: Option<u64>,
			is_following: bool,
			token_id: CurrencyIdOf<T>,
			curve_type: CurveType,
			token_name: Vec<u8>,
			token_decimals: u8,
			token_symbol: Vec<u8>,
			max_supply: BalanceOf<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;

			log::info!(
				"native token free_balance {:#?}",
				T::Currency::free_balance(T::GetNativeCurrencyId::get(), &creator)
			);

			// Requires an amount to be reserved.
			ensure!(
				T::Currency::can_reserve(
					T::GetNativeCurrencyId::get(),
					&creator,
					T::CreatorAssetDeposit::get()
				),
				Error::<T>::InsufficientBalanceToReserve,
			);

			// Ensure that a curve with this id does not already exist.
			ensure!(
				T::Currency::total_issuance(token_id) == 0u32.into(),
				Error::<T>::TokenAlreadyExists,
			);

			log::info!("total issuance {:#?}", T::Currency::total_issuance(token_id));

			// Adds 1 of the token to the module account.
			T::Currency::deposit(
				token_id,
				&T::PalletId::get().into_account(),
				2u128.saturated_into(),
			)?;

			log::info!("total issuance {:#?}", T::Currency::total_issuance(token_id));

			let curr_id = Self::get_next_id();

			let new_token_info = TokenInfo::<T> {
				token_id: token_id.clone(),
				curve_id: curr_id,
				creator: creator.clone(),
				curve_type,
				token_name: token_name.clone(),
				token_symbol,
				token_decimals,
				max_supply,
			};

			let new_user = User::<T> {
				id: curr_id,
				name: user_name.clone(),
				profile_image,
				vines_count,
				is_following,
				accounts: vec![creator],
				token_info: new_token_info,
			};

			Users::<T>::insert(curr_id, new_user.clone());
			TokensStorage::<T>::insert(token_id, new_user);

			Self::deposit_event(Event::UserWithTokenCreated(
				curr_id,
				user_name,
				token_id,
				token_name,
			));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn buy_user_token(
			origin: OriginFor<T>,
			token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			let token = Self::tokens_storage(token_id).ok_or(<Error<T>>::TokenDoesNotExist)?;

			let total_issuance = T::Currency::total_issuance(token_id).saturated_into::<u128>();
			log::info!("total issuance {:#?}", total_issuance.clone());

			let issuance_after = total_issuance + amount.saturated_into::<u128>();
			ensure!(
				issuance_after <= token.token_info.max_supply.saturated_into::<u128>(),
				Error::<T>::TokenMaxSupplyExceeded,
			);

			let curve_config = token.token_info.curve_type.get_curve_config();
			log::info!("curve_config: {:#?}", curve_config);

			let integral_before: BalanceOf<T> =
				curve_config.integral(total_issuance).saturated_into();
			let integral_after: BalanceOf<T> =
				curve_config.integral(issuance_after).saturated_into();

			let cost = integral_after - integral_before;
			log::info!("cost to buy {:#?} tokens is {:#?}", amount, cost.clone());

			ensure!(
				T::Currency::free_balance(T::GetNativeCurrencyId::get(), &buyer) >= cost.into(),
				Error::<T>::InsufficentBalanceForPurchase,
			);

			let token_account = T::PalletId::get().into_sub_account(token.token_info.curve_id);

			// Transfer the network tokens from the buyers' acoount
			// to the admin account
			T::Currency::transfer(T::GetNativeCurrencyId::get(), &buyer, &token_account, cost)?;

			// Deposit the creator tokens to the buyer's acoount
			T::Currency::deposit(token_id, &buyer, amount)?;

			Self::deposit_event(Event::TokenMint(buyer, token_id, amount, cost));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn sell_user_token(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn user_token_spot_price(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn user_token_airdrop(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_id() -> u64 {
		let id = Self::next_id();
		log::info!("before next_id {:#?}", id);
		<NextId<T>>::mutate(|n| *n += 1);
		let id = Self::next_id();
		log::info!("after next_id {:#?}", id);
		id
	}
}
