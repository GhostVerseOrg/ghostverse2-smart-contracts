// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                            6
// Async Callback (empty):               1
// Total number of exported functions:   9

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    ghostversemarketplace
    (
        init => init
        upgrade => upgrade
        getFullMarketplaceData => get_full_marketplace_data
        get_listing => get_listing
        list_nft => list_nft
        buy_nft => buy_nft
        update_price => update_price
        cancel_listing => cancel_listing
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
