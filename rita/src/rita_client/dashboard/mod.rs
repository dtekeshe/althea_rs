//! This file contains all the network endpoints used for the client dashbaord. This management dashboard
//! is for users to use to configure and manage their router and should be firewalled from the outside
//! world.
//!
//! For more documentation on specific functions see the router-dashboard file in the docs folder

use ::actix::prelude::*;
use ::actix_web::http::StatusCode;
use ::actix_web::Path;
use ::actix_web::{AsyncResponder, HttpRequest, HttpResponse, Json};

use failure::Error;

use tokio::timer::Delay;

use serde_json::Value;

use std::boxed::Box;
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use futures::future;
use futures::Future;

use log::LevelFilter;

use reqwest;

use babel_monitor::Babel;

use crate::rita_common::dashboard::Dashboard;
use crate::rita_common::debt_keeper::{DebtKeeper, Dump, NodeDebtData};
use crate::rita_common::peer_listener::PeerListener;
use crate::rita_common::peer_listener::{Listen, UnListen};
use crate::rita_common::tunnel_manager::{GetNeighbors, Neighbor, TunnelManager};
use althea_types::Identity;

use crate::SETTING;
use settings::client::ExitServer;
use settings::client::RitaClientSettings;
use settings::RitaCommonSettings;

use crate::KI;

use num256::Int256;

use crate::rita_client::exit_manager::exit_setup_request;
use althea_types::ExitState;

pub mod exits;
pub mod interfaces;
pub mod logging;
pub mod mesh_ip;
pub mod neighbors;
pub mod system_chain;
pub mod wifi;

use crate::ARGS;
use settings::FileWrite;
