use std::str::FromStr;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use crate::{
    spec::Specification,
    subcommands::{Generate, Print},
};

mod spec;
mod subcommands;
mod proto_gen;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Generate proto files")]
    Generate(Generate),
    #[clap(about = "Print the spec to standard output")]
    Print(Print),
}

#[derive(Debug, Clone)]
struct GenerationProfile {
    version: SpecVersion,
    raw_specs: RawSpecs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpecVersion {
    V0_1_0,
    V0_2_1,
    V0_3_0,
    V0_4_0,
    V0_5_1,
    V0_6_0,
    V0_7_1,
    V0_8_1,
}

#[derive(Debug, Clone)]
struct RawSpecs {
    main: &'static str,
    write: &'static str,
    trace: &'static str,
    ws: Option<&'static str>,
}

impl FromStr for SpecVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "0.1.0" | "v0.1.0" => Self::V0_1_0,
            "0.2.1" | "v0.2.1" => Self::V0_2_1,
            "0.3.0" | "v0.3.0" => Self::V0_3_0,
            "0.4.0" | "v0.4.0" => Self::V0_4_0,
            "0.5.1" | "v0.5.1" => Self::V0_5_1,
            "0.6.0" | "v0.6.0" => Self::V0_6_0,
            "0.7.1" | "v0.7.1" => Self::V0_7_1,
            "0.8.1" | "v0.8.1" => Self::V0_8_1,
            _ => anyhow::bail!("unknown spec version: {}", s),
        })
    }
}

impl ValueEnum for SpecVersion {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::V0_1_0,
            Self::V0_2_1,
            Self::V0_3_0,
            Self::V0_4_0,
            Self::V0_5_1,
            Self::V0_6_0,
            Self::V0_7_1,
            Self::V0_8_1,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        use clap::builder::PossibleValue;

        match self {
            Self::V0_1_0 => Some(PossibleValue::new("0.1.0").alias("v0.1.0")),
            Self::V0_2_1 => Some(PossibleValue::new("0.2.1").alias("v0.2.1")),
            Self::V0_3_0 => Some(PossibleValue::new("0.3.0").alias("v0.3.0")),
            Self::V0_4_0 => Some(PossibleValue::new("0.4.0").alias("v0.4.0")),
            Self::V0_5_1 => Some(PossibleValue::new("0.5.1").alias("v0.5.1")),
            Self::V0_6_0 => Some(PossibleValue::new("0.6.0").alias("v0.6.0")),
            Self::V0_7_1 => Some(PossibleValue::new("0.7.1").alias("v0.7.1")),
            Self::V0_8_1 => Some(PossibleValue::new("0.8.1").alias("v0.8.1")),
        }
    }
}

impl RawSpecs {
    pub fn parse_full(&self) -> Result<Specification> {
        let mut specs: Specification = serde_json::from_str(self.main)?;
        let mut write_specs: Specification = serde_json::from_str(self.write)?;
        let mut trace_specs: Specification = serde_json::from_str(self.trace)?;
        let mut ws_specs: Specification = match &self.ws {
            Some(ws) => serde_json::from_str(ws)?,
            // Pretend spec exists to play nicely with the aggregation code below
            None => Specification {
                openrpc: "mock".into(),
                info: spec::Info {
                    version: "mock".into(),
                    title: "mock".into(),
                    license: spec::Empty {},
                },
                servers: vec![],
                methods: vec![],
                components: spec::Components {
                    content_descriptors: spec::Empty {},
                    schemas: Default::default(),
                    errors: Default::default(),
                },
            },
        };

        for additional_specs in [&mut write_specs, &mut trace_specs, &mut ws_specs].into_iter() {
            specs.methods.append(&mut additional_specs.methods);

            for (key, value) in additional_specs.components.schemas.iter() {
                match specs.components.schemas.entry(key.to_owned()) {
                    indexmap::map::Entry::Occupied(entry) => match &value {
                        spec::Schema::Ref(_) => {}
                        _ => {
                            if value != entry.get() {
                                anyhow::bail!(
                                    "duplicate entries must be ref or identical: {}",
                                    key
                                );
                            }
                        }
                    },
                    indexmap::map::Entry::Vacant(entry) => {
                        entry.insert(value.to_owned());
                    }
                }
            }

            for (key, value) in additional_specs.components.errors.iter() {
                if let indexmap::map::Entry::Vacant(entry) =
                    specs.components.errors.entry(key.to_owned())
                {
                    entry.insert(value.to_owned());
                }
            }
        }

        Ok(specs)
    }
}

fn main() {
    let cli = Cli::parse();

    let profiles: [GenerationProfile; 8] = [
        GenerationProfile {
            version: SpecVersion::V0_1_0,
            raw_specs: RawSpecs {
                main: include_str!("./specs/0.1.0/starknet_api_openrpc.json"),
                write: include_str!("./specs/0.1.0/starknet_write_api.json"),
                trace: include_str!("./specs/0.1.0/starknet_trace_api_openrpc.json"),
                ws: None,
            },
        },
        GenerationProfile {
            version: SpecVersion::V0_2_1,
            raw_specs: RawSpecs {
                main: include_str!("./specs/0.2.1/starknet_api_openrpc.json"),
                write: include_str!("./specs/0.2.1/starknet_write_api.json"),
                trace: include_str!("./specs/0.2.1/starknet_trace_api_openrpc.json"),
                ws: None,
            },
        },
        GenerationProfile {
            version: SpecVersion::V0_3_0,
            raw_specs: RawSpecs {
                main: include_str!("./specs/0.3.0/starknet_api_openrpc.json"),
                write: include_str!("./specs/0.3.0/starknet_write_api.json"),
                trace: include_str!("./specs/0.3.0/starknet_trace_api_openrpc.json"),
                ws: None,
            },
        },
        GenerationProfile {
            version: SpecVersion::V0_4_0,
            raw_specs: RawSpecs {
                main: include_str!("./specs/0.4.0/starknet_api_openrpc.json"),
                write: include_str!("./specs/0.4.0/starknet_write_api.json"),
                trace: include_str!("./specs/0.4.0/starknet_trace_api_openrpc.json"),
                ws: None,
            },
        },
        GenerationProfile {
            version: SpecVersion::V0_5_1,
            raw_specs: RawSpecs {
                main: include_str!("./specs/0.5.1/starknet_api_openrpc.json"),
                write: include_str!("./specs/0.5.1/starknet_write_api.json"),
                trace: include_str!("./specs/0.5.1/starknet_trace_api_openrpc.json"),
                ws: None,
            },
        },
        GenerationProfile {
            version: SpecVersion::V0_6_0,
            raw_specs: RawSpecs {
                main: include_str!("./specs/0.6.0/starknet_api_openrpc.json"),
                write: include_str!("./specs/0.6.0/starknet_write_api.json"),
                trace: include_str!("./specs/0.6.0/starknet_trace_api_openrpc.json"),
                ws: None,
            },
        },
        GenerationProfile {
            version: SpecVersion::V0_7_1,
            raw_specs: RawSpecs {
                main: include_str!("./specs/0.7.1/starknet_api_openrpc.json"),
                write: include_str!("./specs/0.7.1/starknet_write_api.json"),
                trace: include_str!("./specs/0.7.1/starknet_trace_api_openrpc.json"),
                ws: None,
            },
        },
        GenerationProfile {
            version: SpecVersion::V0_8_1,
            raw_specs: RawSpecs {
                main: include_str!("./specs/0.8.1/starknet_api_openrpc.json"),
                write: include_str!("./specs/0.8.1/starknet_write_api.json"),
                trace: include_str!("./specs/0.8.1/starknet_trace_api_openrpc.json"),
                ws: Some(include_str!("./specs/0.8.1/starknet_ws_api.json")),
            },
        },
    ];

    let result = match cli.command {
        Subcommands::Generate(cmd) => cmd.run(&profiles),
        Subcommands::Print(cmd) => cmd.run(&profiles),
    };

    result.expect("Error running commmand");
}
