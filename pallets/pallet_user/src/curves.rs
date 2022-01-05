use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct CurveConfig {
	exponent: u32,
	slope: u128,
}

impl Default for CurveConfig {
	fn default() -> Self {
		Self { exponent: 1, slope: 1 }
	}
}

impl CurveConfig {
	pub fn integral(&self, x: u128) -> u128 {
		let nexp = self.exponent + 1;
		let val = x.pow(nexp) * self.slope / nexp as u128;
		log::info!("nexp = {:?} slope = {:?} exponent = {:?}", nexp, self.slope, self.exponent);
		log::info!("Integral value {:?}", val);
		return val;
	}
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum CurveType {
	Linear,
	Exponential,
	Flat,
	Logarithmic,
}

impl CurveType {
	pub fn get_curve_config(&self) -> CurveConfig {
		match &self {
			CurveType::Exponential => CurveConfig { exponent: 10, slope: 10 },
			CurveType::Flat => CurveConfig { exponent: 100, slope: 100 },
			CurveType::Linear => CurveConfig { exponent: 50, slope: 100 },
			CurveType::Logarithmic => CurveConfig { exponent: 90, slope: 100 },
		}
	}
}
