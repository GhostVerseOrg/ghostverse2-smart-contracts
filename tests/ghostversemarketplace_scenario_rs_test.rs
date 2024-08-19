use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/ghostversemarketplace.mxsc.json", ghostversemarketplace::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/ghostversemarketplace.scen.json");
}