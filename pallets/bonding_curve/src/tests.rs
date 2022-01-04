use crate::{curves::CurveType, mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use frame_system::{ensure_signed, RawOrigin};

#[test]
fn correct_error_for_unsigned_origin_while_creating_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::create_asset(
				Origin::none(),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			),
			DispatchError::BadOrigin,
		);
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 0);
	});
}

#[test]
fn correct_error_for_insufficient_balance_to_reserve_while_creating_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::create_asset(
				Origin::signed(1),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			),
			Error::<Test>::InsufficientBalanceToReserve,
		);
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 0);
	});
}

#[test]
fn correct_error_for_asset_already_existing_while_creating_asset() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_noop!(
			PalletYodaBondingCurve::create_asset(
				Origin::signed(2),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			),
			Error::<Test>::AssetAlreadyExists,
		);
	});
}

#[test]
fn it_works_for_create_asset_with_correct_parameters() {
	let mut extrinsic = ExtBuilder::default().with_balances(vec![(1, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 0);
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 1);
		assert_eq!(
			PalletYodaBondingCurve::assets_minted(10),
			Some(crate::BondingToken::<Test> {
				creator: 1,
				asset_id: 10,
				curve: CurveType::Linear,
				max_supply: 10000,
				token_name: b"batman".to_vec(),
				token_symbol: b"bat".to_vec(),
				token_decimals: 10,
				curve_id: 0,
				mint_data: crate::MintingData::<Test> {
					minter: 1,
					minting_cap: Some(10000),
					current_mint_amount: Some(2000),
				},
			})
		);
	});
}

#[test]
fn correct_error_for_unsigned_origin_while_minting_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::create_asset(
				Origin::none(),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			),
			DispatchError::BadOrigin,
		);
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 0);
	});
}

#[test]
fn correct_error_for_asset_does_not_exist_while_minting_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::mint_asset(Origin::signed(1), 10, 10000,),
			Error::<Test>::AssetDoesNotExist,
		);
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 0);
	});
}

#[test]
fn correct_error_for_invalid_minter_while_minting_asset() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 2000000)]).build();
	extrinsic.execute_with(|| {
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 0);
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 1);
		assert_noop!(
			PalletYodaBondingCurve::mint_asset(Origin::signed(2), 10, 10000,),
			Error::<Test>::InvalidMinter,
		);
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 1);
	});
}

#[test]
fn it_works_for_mint_asset_with_correct_parameters() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 2000000)]).build();
	extrinsic.execute_with(|| {
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 0);
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 1);
		assert_eq!(
			PalletYodaBondingCurve::assets_minted(10),
			Some(crate::BondingToken::<Test> {
				creator: 1,
				asset_id: 10,
				curve: CurveType::Linear,
				max_supply: 10000,
				token_name: b"batman".to_vec(),
				token_symbol: b"bat".to_vec(),
				token_decimals: 10,
				curve_id: 0,
				mint_data: crate::MintingData::<Test> {
					minter: 1,
					minting_cap: Some(10000),
					current_mint_amount: Some(2000),
				},
			})
		);
		assert_ok!(PalletYodaBondingCurve::mint_asset(Origin::signed(1), 10, 10000));
		assert_eq!(
			PalletYodaBondingCurve::assets_minted(10),
			Some(crate::BondingToken::<Test> {
				creator: 1,
				asset_id: 10,
				curve: CurveType::Linear,
				max_supply: 10000,
				token_name: b"batman".to_vec(),
				token_symbol: b"bat".to_vec(),
				token_decimals: 10,
				curve_id: 0,
				mint_data: crate::MintingData::<Test> {
					minter: 1,
					minting_cap: Some(10000),
					current_mint_amount: Some(12000),
				},
			})
		);
		assert_eq!(crate::AssetsMinted::<Test>::iter().count(), 1);
	});
}

#[test]
fn correct_error_while_buying_an_asset_that_does_not_exist() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::buy_asset(Origin::signed(1), 30, 3000,),
			Error::<Test>::AssetDoesNotExist,
		);
	});
}

#[test]
fn correct_error_while_buying_an_asset_whose_amount_is_greater_than_the_current_token_supply() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_noop!(
			PalletYodaBondingCurve::buy_asset(Origin::signed(2), 10, 3000,),
			Error::<Test>::MintAmountGreaterThanMaxSupply,
		);
	});
}

#[test]
fn correct_error_while_buying_an_asset_when_the_buyer_has_insufficient_balance_of_the_network_token(
) {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 100000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_noop!(
			PalletYodaBondingCurve::buy_asset(Origin::signed(2), 10, 1500),
			pallet_balances::pallet::Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn it_works_for_buying_as_asset_with_correct_parameters() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_ok!(PalletYodaBondingCurve::buy_asset(Origin::signed(2), 10, 100));
	});
}

#[test]
fn correct_error_for_unsigned_origin_while_selling_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::sell_asset(
				Origin::none(),
				10,
				ensure_signed(Origin::signed(2)).unwrap(),
				5000,
			),
			DispatchError::BadOrigin,
		);
	});
}

#[test]
fn correct_error_while_selling_an_asset_that_does_not_exist() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::sell_asset(
				Origin::signed(1),
				10,
				ensure_signed(Origin::signed(2)).unwrap(),
				5000,
			),
			Error::<Test>::AssetDoesNotExist,
		);
	});
}

#[test]
fn correct_error_when_the_seller_sells_for_amount_more_than_in_the_network() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 2000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_noop!(
			PalletYodaBondingCurve::sell_asset(
				Origin::signed(1),
				10,
				ensure_signed(Origin::signed(2)).unwrap(),
				1000000,
			),
			orml_tokens::Error::<Test>::BalanceTooLow
		);
	});
}

#[test]
fn it_works_for_sell_asset_with_correct_parameters() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 2000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_ok!(PalletYodaBondingCurve::sell_asset(
			Origin::signed(1),
			10,
			ensure_signed(Origin::signed(2)).unwrap(),
			1000,
		),);
	});
}

// #[test]
// fn correct_error_for_unsigned_origin_while_air_drop() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(PalletYodaBondingCurve::create_asset(
// 			Origin::signed(1),
// 			10,
// 			10000,
// 			CurveType::Linear,
// 			2000,
// 			b"batman".to_vec(),
// 			b"bat".to_vec(),
// 			10
// 		));
// 		assert_noop!(
// 			PalletYodaBondingCurve::air_drop(
// 				Origin::none(),
// 				10,
// 				vec![
// 					Origin::signed(2),
// 					Origin::signed(3),
// 					Origin::signed(4),
// 					Origin::signed(5),
// 				],
// 				500,
// 			),
// 			DispatchError::BadOrigin,
// 		);
// 	});
// }