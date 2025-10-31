use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DexEnum {
    Woofi,
    PumpFun,
    Whirlpool,
    Virtuals,
    DaosFun,
    LifinityV2,
    StabbleStableSwap,
    TokenMill,
    Meteora,
    Oasis,
    Aldrin,
    GooseFxGamma,
    Perps,
    SolFi,
    DexLab,
    TokenSwap,
    ZeroFi,
    Cropper,
    ObricV2,
    StabbleWeightedSwap,
    SanctumInfinity,
    Moonit,
    Sanctum,
    RaydiumCp,
    Phoenix,
    PumpFunAmm,
    Saber,
    SaberDecimals,
    RaydiumClmm,
    Dex1,
    Penguin,
    OrcaV2,
    FluxBeam,
    Raydium,
    MeteoraDlmm,
    Bonkswap,
    Solayer,
    Stepn,
    HeliumNetwork,
    Mercurial,
    Perena,
    OrcaV1,
    AldrinV2,
    Saros,
    OpenBookV2,
    Crema,
    OpenBook,
    Invariant,
    Guacswap,
}

impl std::fmt::Display for DexEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            DexEnum::Woofi => "Woofi",
            DexEnum::PumpFun => "Pump.fun",
            DexEnum::Whirlpool => "Whirlpool",
            DexEnum::Virtuals => "Virtuals",
            DexEnum::DaosFun => "Daos.fun",
            DexEnum::LifinityV2 => "Lifinity V2",
            DexEnum::StabbleStableSwap => "Stabble Stable Swap",
            DexEnum::TokenMill => "Token Mill",
            DexEnum::Meteora => "Meteora",
            DexEnum::Oasis => "Oasis",
            DexEnum::Aldrin => "Aldrin",
            DexEnum::GooseFxGamma => "GooseFX GAMMA",
            DexEnum::Perps => "Perps",
            DexEnum::SolFi => "SolFi",
            DexEnum::DexLab => "DexLab",
            DexEnum::TokenSwap => "Token Swap",
            DexEnum::ZeroFi => "ZeroFi",
            DexEnum::Cropper => "Cropper",
            DexEnum::ObricV2 => "Obric V2",
            DexEnum::StabbleWeightedSwap => "Stabble Weighted Swap",
            DexEnum::SanctumInfinity => "Sanctum Infinity",
            DexEnum::Moonit => "Moonit",
            DexEnum::Sanctum => "Sanctum",
            DexEnum::RaydiumCp => "Raydium CP",
            DexEnum::Phoenix => "Phoenix",
            DexEnum::PumpFunAmm => "Pump.fun Amm",
            DexEnum::Saber => "Saber",
            DexEnum::SaberDecimals => "Saber (Decimals)",
            DexEnum::RaydiumClmm => "Raydium CLMM",
            DexEnum::Dex1 => "1DEX",
            DexEnum::Penguin => "Penguin",
            DexEnum::OrcaV2 => "Orca V2",
            DexEnum::FluxBeam => "FluxBeam",
            DexEnum::Raydium => "Raydium",
            DexEnum::MeteoraDlmm => "Meteora DLMM",
            DexEnum::Bonkswap => "Bonkswap",
            DexEnum::Solayer => "Solayer",
            DexEnum::Stepn => "StepN",
            DexEnum::HeliumNetwork => "Helium Network",
            DexEnum::Mercurial => "Mercurial",
            DexEnum::Perena => "Perena",
            DexEnum::OrcaV1 => "Orca V1",
            DexEnum::AldrinV2 => "Aldrin V2",
            DexEnum::Saros => "Saros",
            DexEnum::OpenBookV2 => "OpenBook V2",
            DexEnum::Crema => "Crema",
            DexEnum::OpenBook => "Openbook",
            DexEnum::Invariant => "Invariant",
            DexEnum::Guacswap => "Guacswap",
        };
        write!(f, "{label}")
    }
}

pub fn dex_vec_to_comma_string<S>(
    vec: &Option<Vec<DexEnum>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match vec {
        Some(v) => {
            let joined = v
                .iter()
                .map(|dex| dex.to_string())
                .collect::<Vec<_>>()
                .join(",");
            serializer.serialize_str(&joined)
        }
        None => serializer.serialize_none(),
    }
}
