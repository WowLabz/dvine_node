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
	use scale_info::prelude::vec;
	use scale_info::{prelude::boxed::Box, TypeInfo};
	use sp_runtime::traits::{AccountIdConversion, CheckedAdd, SaturatedConversion};

	type BalanceOf<T> =
		<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;
	pub type UserId = u64;

	#[derive(Encode, Decode, Debug, TypeInfo, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct User<T: Config> {
		pub id: UserId,
		pub name: Vec<u8>,
		pub profile_image: Vec<u8>,
		pub vines_count: Option<u64>,
		pub is_following: bool,
		pub accounts: Vec<AccountOf<T>>,
		pub token_info: Option<TokenInfo<T>>,
	}

	#[derive(Encode, Decode, Debug, TypeInfo, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct TokenInfo<T: Config> {
		pub token_id: CurrencyIdOf<T>,
		pub curve_id: u64,
		pub creator: AccountOf<T>,
		pub curve_type: CurveType,
		pub token_name: Vec<u8>,
		pub token_symbol: Vec<u8>,
		pub token_decimals: u8,
		pub max_supply: BalanceOf<T>,
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
	#[pallet::getter(fn next_token_id)]
	pub(super) type NextTokenId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_user_id)]
	pub(super) type NextUserId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn users)]
	pub type Users<T: Config> = StorageMap<_, Twox64Concat, UserId, User<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_storage)]
	pub type TokenStorage<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, User<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_spot_price)]
	pub(super) type TokenSpotPrice<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [UserId, UserName]
		UserCreated(UserId, Vec<u8>),
		/// [UserId, UserName, TokenId, TokenName]
		UserWithTokenCreated(UserId, Vec<u8>, CurrencyIdOf<T>, Vec<u8>),
		/// (Minter, TokenId, MintAmount, Cost)
		TokenMint(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// (Burner, TokenId, BurnAmount, ReturnAmount)
		TokenBurn(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// (TokenId, Amount)
		TokenSpotPrice(CurrencyIdOf<T>, BalanceOf<T>),
		/// (TokenId, Amount, FromAccount, ToAccounts)
		UserTokensAirDropped(CurrencyIdOf<T>, BalanceOf<T>, AccountOf<T>, Vec<AccountOf<T>>),
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
		/// User does not exist
		UserDoesNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_user(
			origin: OriginFor<T>,
			user_name: Vec<u8>,
			profile_image: Vec<u8>,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;

			let curr_user_id = Self::get_next_user_id();

			let new_user = User::<T> {
				id: curr_user_id,
				name: user_name.clone(),
				profile_image,
				vines_count: None,
				is_following: false,
				accounts: vec![creator],
				token_info: None,
			};

			Users::<T>::insert(curr_user_id, new_user.clone());

			Self::deposit_event(Event::<T>::UserCreated(curr_user_id, user_name));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_user_token(
			origin: OriginFor<T>,
			user_id: UserId,
			token_id: CurrencyIdOf<T>,
			curve_type: CurveType,
		 	token_name: Vec<u8>,
		 	token_symbol: Vec<u8>,
		 	token_decimals: u8,
		 	max_supply: BalanceOf<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;

			let mut user = Self::users(user_id).ok_or(<Error<T>>::UserDoesNotExist)?;

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

			let curr_curve_id = Self::get_next_token_id();

			// Ensure that a curve with this id does not already exist.
			ensure!(
				T::Currency::total_issuance(token_id) == 0u32.into(),
				Error::<T>::TokenAlreadyExists,
			);

			// Adds 1 of the token to the module account.
			T::Currency::deposit(
				token_id,
				&T::PalletId::get().into_account(),
				2u128.saturated_into(),
			)?;

			log::info!("total issuance {:#?}", T::Currency::total_issuance(token_id));

			let new_token_info = TokenInfo::<T> {
				token_id: token_id.clone(), 
				curve_id: curr_curve_id,
				creator,
				curve_type,
				token_name: token_name.clone(),
				token_symbol,
				token_decimals,
				max_supply,
			};

			// Update the user with token_info
			// and update the storages
			user.token_info = Some(new_token_info);
			Users::<T>::insert(user_id.clone(), user.clone());
			TokenStorage::<T>::insert(token_id.clone(), user.clone());

			Self::deposit_event(Event::UserWithTokenCreated(
				user_id, user.name, token_id, token_name,
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

			let token = Self::token_storage(token_id).ok_or(<Error<T>>::TokenDoesNotExist)?;
			let curr_token_info = token.token_info.ok_or(Error::<T>::TokenDoesNotExist)?;

			let total_issuance = T::Currency::total_issuance(token_id).saturated_into::<u128>();
			log::info!("total issuance {:#?}", total_issuance.clone());

			let issuance_after = total_issuance + amount.saturated_into::<u128>();
			ensure!(
				issuance_after <= curr_token_info.max_supply.saturated_into::<u128>(),
				Error::<T>::TokenMaxSupplyExceeded,
			);

			let curve_config = curr_token_info.curve_type.get_curve_config();
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

			let token_account = T::PalletId::get().into_sub_account(curr_token_info.curve_id);

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
			token_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;

			let token = Self::token_storage(token_id).ok_or(<Error<T>>::TokenDoesNotExist)?;
			let curr_token_info = token.token_info.ok_or(Error::<T>::TokenDoesNotExist)?;

			T::Currency::ensure_can_withdraw(token_id, &seller, amount)?;

			let total_issuance = T::Currency::total_issuance(token_id);
			let issuance_after = total_issuance - amount;

			let curve_config = curr_token_info.curve_type.get_curve_config();
			log::info!("curve_config: {:#?}", curve_config);

			let integral_before: BalanceOf<T> =
				curve_config.integral(total_issuance.saturated_into::<u128>()).saturated_into();
			let integral_after: BalanceOf<T> =
				curve_config.integral(issuance_after.saturated_into::<u128>()).saturated_into();

			let return_amount = integral_before - integral_after;
			log::info!(
				"return amount selling {:#?} tokens is {:#?}",
				amount,
				return_amount.clone()
			);

			let token_account = T::PalletId::get().into_sub_account(curr_token_info.curve_id);

			T::Currency::withdraw(token_id, &seller, amount)?;

			T::Currency::transfer(
				T::GetNativeCurrencyId::get(),
				&token_account,
				&seller,
				return_amount,
			)?;

			Self::deposit_event(Event::TokenBurn(seller, token_id, amount, return_amount));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn user_token_spot_price(
			origin: OriginFor<T>,
			token_id: CurrencyIdOf<T>,
		) -> DispatchResult {
			let _user = ensure_signed(origin)?;

			let token = Self::token_storage(token_id).ok_or(<Error<T>>::TokenDoesNotExist)?;
			let curr_token_info = token.token_info.ok_or(Error::<T>::TokenDoesNotExist)?;
			let curve_config = curr_token_info.curve_type.get_curve_config();
			let total_issuance = T::Currency::total_issuance(token_id).saturated_into::<u128>();
			log::info!("Total Issuance of the asset {:?}", total_issuance);

			let current_price: u128 = curve_config.integral(total_issuance);
			let spot_price: BalanceOf<T> = (current_price / total_issuance).saturated_into();
			log::info!("spot price: {:#?}", current_price.clone());
			log::info!(
				"actual spot price{:?}",
				current_price.clone().saturated_into::<u128>() / total_issuance.clone()
			);

			<TokenSpotPrice<T>>::insert(token_id.clone(), spot_price);

			Self::deposit_event(Event::TokenSpotPrice(token_id, spot_price));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn user_token_airdrop(
			origin: OriginFor<T>,
			token_id: CurrencyIdOf<T>,
			beneficiaries: Vec<AccountOf<T>>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;

			let token = Self::token_storage(token_id).ok_or(<Error<T>>::TokenDoesNotExist)?;
			let curr_token_info = token.token_info.ok_or(Error::<T>::TokenDoesNotExist)?;

			let total_withdraw_amount: BalanceOf<T> =
				(amount.saturated_into::<u128>() * beneficiaries.len() as u128).saturated_into();
			log::info!("airdrop total_withdraw_amount: {:#?}", total_withdraw_amount);

			for beneficiary in &beneficiaries {
				T::Currency::deposit(token_id, beneficiary, amount)?;
			}

			Self::deposit_event(Event::UserTokensAirDropped(
				token_id,
				amount,
				curr_token_info.creator,
				beneficiaries.clone(),
			));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_next_token_id() -> u64 {
			let id = Self::next_token_id();
			log::info!("before next_id {:#?}", id);
			<NextTokenId<T>>::mutate(|n| *n += 1);
			let id = Self::next_token_id();
			log::info!("after next_id {:#?}", id);
			id
		}

		fn get_next_user_id() -> u64 {
			<NextUserId<T>>::mutate(|n| *n += 1);
			Self::next_user_id()
		}
	}
}
