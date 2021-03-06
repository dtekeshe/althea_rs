use super::{
    clients_to_ids, to_exit_client, DbClient, Future, Instant, ListClients, RitaCommonSettings,
    RitaExitSettings, SystemService, TrafficWatcher, Watch, KI, SETTING,
};

/// Takes data from the exit database and uses it to create a series of exit tunnels
/// this will setup every user in the database right off the bat and monitor them as
/// one huge wg_exit tunnel definition. It will also rerun the commands to define this
/// tunnel every round, which isn't great from an efficiency point of view but is somthing
/// that wireguard explicitly allows in terms of non-traffic disrupting operations.
pub fn setup_exit_clients() -> Box<Future<Item = (), Error = ()>> {
    let start = Instant::now();
    Box::new(
        DbClient::from_registry()
            .send(ListClients {})
            .then(move |res| {
                let clients = res.unwrap().unwrap();
                let ids = clients_to_ids(clients.clone());

                // watch and bill for traffic
                TrafficWatcher::from_registry().do_send(Watch(ids));

                let mut wg_clients = Vec::new();

                trace!("got clients from db {:?}", clients);

                for c in clients {
                    if let Ok(c) = to_exit_client(c) {
                        wg_clients.push(c);
                    }
                }

                trace!("converted clients {:?}", wg_clients);

                // setup all the tunnels
                let exit_status = KI.set_exit_wg_config(
                    wg_clients,
                    SETTING.get_exit_network().wg_tunnel_port,
                    &SETTING.get_network().wg_private_key_path,
                    &SETTING.get_exit_network().own_internal_ip,
                    SETTING.get_exit_network().netmask,
                );

                match exit_status {
                    Ok(_) => (),
                    Err(e) => warn!("Error in Exit WG setup {:?}", e),
                }
                info!(
                    "Rita Exit loop completed in {}s {}ms",
                    start.elapsed().as_secs(),
                    start.elapsed().subsec_millis()
                );
                Ok(())
            }),
    )
}
