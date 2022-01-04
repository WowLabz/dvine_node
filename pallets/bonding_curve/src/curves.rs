use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::Scale, SaturatedConversion};

// #[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
// #[cfg_attr(feature = "std", derive(Debug))]
// pub enum CurveType {
// 	Linear,
// 	Exponential,
// 	Flat,
// 	Logarithmic,
// }

// impl CurveType {
// 	pub fn get_reserve_ratio(&self) -> (u128, u128) {
// 		match &self {
// 			CurveType::Exponential => (10, 100),
// 			CurveType::Flat => (100, 100),
// 			CurveType::Linear => (50, 100),
// 			CurveType::Logarithmic => (90, 100),
// 		}
// 	}
// }

pub trait CurveConfig {
	fn integral_before(&self, issuance: u128) -> u128;
	fn integral_after(&self, issuance: u128) -> u128;
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum CurveType {
	Linear,
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Linear {
	exponent: u32,
	slope: u128,
}

impl Linear {
	pub fn new(exponent: u32, slope: u128) -> Self {
		Self { exponent, slope }
	}
	/// Integral when the curve is at point `x`.
	pub fn integral(&self, x: u128) -> u128 {
		let nexp = self.exponent + 1;
		let val = x.pow(nexp) * self.slope / nexp as u128;
		log::info!("nexp = {:?} slope = {:?} exponent = {:?}", nexp, self.slope, self.exponent);
		log::info!("Integral value {:?}", val);
		return val
	}
}

impl Default for Linear {
	fn default() -> Self {
		Self { exponent: 1, slope: 1 }
	}
}

impl CurveConfig for Linear {
	fn integral_before(&self, issuance: u128) -> u128 {
		self.integral(issuance).saturated_into()
	}
	fn integral_after(&self, issuance: u128) -> u128 {
		self.integral(issuance).saturated_into()
	}
}