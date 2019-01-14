use super::*;

use settings::payment::SystemChain;

/// Changes the full node configuration value between test/prod and other networks
pub fn set_system_blockchain(
    path: Path<SystemChain>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    info!("Blockchain change endpoint hit!");
    let id = path.into_inner();
    let mut payment = SETTING.get_payment_mut();
    let mut dao = SETTING.get_dao_mut();

    if id == SystemChain::Ethereum {
        payment.node_list = vec![
            "https://eth.althea.org:443".to_string(),
            "https://mainnet.infura.io/v3/6b080f02d7004a8394444cdf232a7081".to_string(),
        ];
        payment.net_version = Some(1);
        payment.system_chain = SystemChain::Ethereum;
        dao.node_list = vec![
            "https://eth.althea.org:443".to_string(),
            "https://mainnet.infura.io/v3/6b080f02d7004a8394444cdf232a7081".to_string(),
        ];
    } else if id == SystemChain::Xdai {
        payment.node_list = vec!["https://dai.poa.network/".to_string()];
        payment.net_version = Some(100);
        payment.system_chain = SystemChain::Xdai;
        dao.node_list = vec!["https://dai.poa.network/".to_string()];
        payment.price_oracle_url = "https://updates.altheamesh.com/xdaiprices".to_string();
    } else if id == SystemChain::Rinkeby {
        payment.node_list = vec!["http://rinkeby.althea.org:8545".to_string()];
        payment.net_version = Some(4);
        payment.system_chain = SystemChain::Rinkeby;
        dao.node_list = vec!["http://rinkeby.althea.org:8545".to_string()];
    } else {
        return Box::new(future::ok(
            HttpResponse::new(StatusCode::BAD_REQUEST)
                .into_builder()
                .json(format!("No known chain by the identifier {:?}", id)),
        ));
    }

    Box::new(future::ok(HttpResponse::Ok().json(())))
}

pub fn get_system_blockchain(
    _req: HttpRequest,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    debug!("/blockchain/ GET hit");

    Box::new(future::ok(
        HttpResponse::Ok().json(SETTING.get_payment().system_chain.clone()),
    ))
}
