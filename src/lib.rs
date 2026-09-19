pub mod utils;
pub mod subprocesses;

pub use clap::{Parser, ArgAction};
pub use std::thread::{available_parallelism, spawn};
pub use std::process::{Command, Stdio, Output};
pub use std::path::{PathBuf, Path};
pub use std::io::stdin;
pub use aws_config::{load_defaults, SdkConfig, BehaviorVersion};
pub use aws_sdk_sts::Client as sts_Client;
pub use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentity;
pub use aws_sdk_ec2::Client;
pub use aws_sdk_ec2::types::{Placement, builders::PlacementBuilder, ShutdownBehavior, InstanceType, Instance};
pub use aws_sdk_ec2::operation::{run_instances::RunInstancesOutput, 
                        describe_images::builders::DescribeImagesFluentBuilder, describe_images::DescribeImagesOutput};
pub use terminal_hyperlink::Hyperlink;
pub use colored::Colorize;
pub use bs58::decode;
pub use base64::Engine;
pub use base64::engine::general_purpose::STANDARD;
pub use std::fs::read_to_string;
pub use std::error::Error;
// pub use core::io::Error;
pub use std::sync::Arc;
pub use std::ffi::OsStr;
pub use std::env;