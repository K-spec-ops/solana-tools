pub mod utils;

pub use clap::{Parser, ArgAction};
pub use std::thread::{available_parallelism, spawn};
pub use std::process::{Command, Stdio, Output};
pub use std::path::{PathBuf, Path};
pub use std::fs::{read, read_to_string, File, OpenOptions};
pub use std::io::{stdin, BufWriter};
pub use aws_config::{load_defaults, SdkConfig, BehaviorVersion};
pub use aws_sdk_iam::Client as iam_Client;
pub use aws_sdk_iam::operation::{get_user::GetUserOutput, create_policy::CreatePolicyOutput};
pub use aws_sdk_s3::primitives::ByteStream;
pub use aws_sdk_s3::operation::put_object::PutObjectOutput;
pub use aws_sdk_s3::Client as s3_Client;
pub use aws_sdk_sts::Client as sts_Client; 
pub use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentity;
pub use aws_sdk_ec2::{Client, client::Waiters};
pub use aws_sdk_ec2::types::{Placement, builders::PlacementBuilder, ShutdownBehavior, InstanceType, Instance};
pub use aws_sdk_ec2::operation::{run_instances::RunInstancesOutput, 
                        describe_images::builders::DescribeImagesFluentBuilder, 
                        describe_images::DescribeImagesOutput, describe_instances::DescribeInstancesOutput};
pub use terminal_hyperlink::Hyperlink;
pub use colored::Colorize;
pub use simplelog::{WriteLogger, TermLogger, ConfigBuilder, TerminalMode, ColorChoice, Config, LevelFilter, Level, Color};
pub use bs58::decode;
pub use uuid::Uuid;
pub use jiff::Zoned;
pub use base64::Engine;
pub use base64::engine::general_purpose::STANDARD;
pub use std::ffi::os_str::Display;
pub use std::time::{Duration, Instant, SystemTime};
pub use std::borrow::Cow;
pub use std::error::Error;
pub use std::fmt::Write;
// pub use core::io::Error;
pub use rand::random_range;
pub use std::sync::Arc;
pub use std::ffi::OsStr;
pub use std::env;