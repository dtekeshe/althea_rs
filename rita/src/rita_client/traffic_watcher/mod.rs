use actix::prelude::*;

use althea_kernel_interface;
use althea_kernel_interface::KernelInterface;
use althea_kernel_interface::FilterTarget;

use althea_types::Identity;

use babel_monitor;
use babel_monitor::Babel;

use rita_common::debt_keeper;
use rita_common::debt_keeper::DebtKeeper;

use futures::{future, Future};

use num256::Int256;

use eui48::MacAddress;

use std::net::{IpAddr, Ipv6Addr};
use std::collections::HashMap;

use ip_network::IpNetwork;

use std::{thread, time};

use SETTING;

use failure::Error;
use althea_types::PaymentTx;
use rita_common::payment_controller::{MakePayment, PaymentController};

pub struct TrafficWatcher;

impl Actor for TrafficWatcher {
    type Context = Context<Self>;
}
impl Supervised for TrafficWatcher {}
impl SystemService for TrafficWatcher {
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        let ki = KernelInterface {};

        info!("Client traffic watcher started");

        ki.init_exit_client_counters();
    }
}
impl Default for TrafficWatcher {
    fn default() -> TrafficWatcher {
        TrafficWatcher {}
    }
}

#[derive(Message)]
pub struct Watch(pub Identity, pub u64);

impl Handler<Watch> for TrafficWatcher {
    type Result = ();

    fn handle(&mut self, msg: Watch, _: &mut Context<Self>) -> Self::Result {
        watch(msg.0, msg.1);
    }
}

/// This traffic watcher watches how much traffic we send to the exit, and how much the exit sends
/// back to us.
pub fn watch(exit: Identity, exit_price: u64) -> Result<(), Error> {
    let ki = KernelInterface {};
    let mut babel = Babel::new(
        &format!("[::1]:{}", SETTING.read().unwrap().network.babel_port)
            .parse()
            .unwrap(),
    );

    trace!("Getting routes");
    let routes = babel.parse_routes()?;
    info!("Got routes: {:?}", routes);

    let mut destinations = HashMap::new();

    for route in &routes {
        // Only ip6
        if let IpNetwork::V6(ref ip) = route.prefix {
            // Only host addresses and installed routes
            if ip.get_netmask() == 128 && route.installed {
                destinations.insert(
                    IpAddr::V6(ip.get_network_address()),
                    Int256::from(route.price),
                );
            }
        }
    }

    let input = ki.read_exit_client_counters_input();
    let output = ki.read_exit_client_counters_output();

    trace!("got {:?} from client exit counters", (&input, &output));

    let input = input?;
    let output = output?;

    let mut owes: Int256 = Int256::from(0);

    let local_price = babel.local_price().unwrap();

    trace!("exit price {}", exit_price);
    trace!(
        "exit destination price {}",
        destinations[&exit.mesh_ip].clone() + exit_price
    );

    owes += Int256::from(exit_price * output);

    owes += (destinations[&exit.mesh_ip].clone() + exit_price) * input;

    let update = debt_keeper::TrafficUpdate {
        from: exit.clone(),
        amount: owes,
    };

    let adjustment = debt_keeper::SendUpdate { from: exit };

    Arbiter::handle().spawn(DebtKeeper::from_registry().send(update).then(move |_| {
        DebtKeeper::from_registry().do_send(adjustment);
        future::result(Ok(()))
    }));
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}