#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// Default values for the marketplace logic.
// We sell single NFT objects only and take 3% marketplace fee and royalties to the creator.
const NFT_AMOUNT: u32 = 1; // We sell single NFT objects only.
const MARKETPLACE_CUT: u32 = 300; // 3%.
pub const PERCENTAGE_TOTAL: u64 = 10_000; // 100%.

// Store the NFT listing objects in the storage.
#[derive(TypeAbi, TopEncode, TopDecode, ManagedVecItem, NestedEncode, NestedDecode)]
pub struct NftListing<M: ManagedTypeApi> {
    pub nft_token: TokenIdentifier<M>, // Collection identifier, e.g. name + 6 random symbols, e.g. <GHOSTSET-531aff>-01.
    pub nft_nonce: u64, // nonce, id for NFT in collection, e.g. GHOSTSET-531aff-<01>.
    pub nft_original_owner: ManagedAddress<M>, // The owner of the NFT who this listing belongs to.
    pub listing_amount: BigUint<M>, // Price of the NFT listing.
    pub listing_publish_time: u64, // Store the publish data of the listing for sorting.
}

#[multiversx_sc::contract]
pub trait Ghostversemarketplace{
    // We must have init function in order for SC to build.
    #[init]
    fn init(&self) {}

    // We reserve the Upgrade function for later use.
    #[upgrade]
    fn upgrade(&self) {}

    /* ________________________________ */
    /* Data mapper and data fetch logic */

    // We must map our data structs explicitly, we map NFTs by unique token+nonce pair for internal NFT listing data.
    #[storage_mapper("listingDetails")]
    fn listing_details(&self) -> MapMapper<(TokenIdentifier, u64), NftListing<Self::Api>>;

    // Get all NFT listings from SC memory, iterate over the storage and return all the data.
    #[view(getFullMarketplaceData)]
    fn get_full_marketplace_data(&self) 
    -> MultiValueManagedVec<NftListing<Self::Api>> {
        let storageDetails = self.listing_details(); // Store results from SC storage in variable to extract iterator from it.
        let storageIterator = storageDetails.values(); // Extract the iterator to loop over the storage values.
        let mut listingsFound: MultiValueManagedVec<Self::Api, NftListing<Self::Api>> = MultiValueManagedVec::new();

        // Iterate through all available data and return everything.
        for listing in storageIterator {
            listingsFound.push(
                NftListing{
                nft_token: listing.nft_token,
                nft_nonce: listing.nft_nonce,
                nft_original_owner: listing.nft_original_owner,
                listing_amount: listing.listing_amount,
                listing_publish_time: listing.listing_publish_time,
                }
            )
        }

        return listingsFound;
    }

    /* _________________________ */
    /* Marketplace functionality */

    // List NFT for sale in the marketplace.
    #[payable("*")]
    #[endpoint(list_nft)]
    fn list_nft(
        &self,
        nft_token: TokenIdentifier,
        nft_nonce: u64,
        listing_amount: BigUint,
    ) -> SCResult<()> {
        let creator_royalties_percentage = self.get_nft_info(&nft_token, nft_nonce).royalties;

        require!(
            BigUint::from(MARKETPLACE_CUT) + &creator_royalties_percentage < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );

        self.listing_details().insert((nft_token.clone(), nft_nonce.clone()), NftListing{
            nft_token: nft_token,
            nft_nonce: nft_nonce,
            nft_original_owner: self.blockchain().get_caller(),
            listing_amount: listing_amount,
            listing_publish_time: self.blockchain().get_block_timestamp(),
        });

        SCResult::Ok(())
    }

    // // endpoints
    // #[payable("EGLD")]
    // #[allow(clippy::too_many_arguments)]
    // #[endpoint(buy_nft)]
    // fn buy_nft(
    //     &self,
    //     #[payment_amount] payment_amount: BigUint,
    //     nft_token_id: TokenIdentifier,
    //     nft_nonce: u64,
    // ) -> SCResult<()> {

    //     // Check the mapped value exists.
    //     require!(
    //         self.nft_detail().contains_key(&(nft_token_id.clone(), nft_nonce.clone())),
    //         "Invalid NFT token, nonce or NFT was already sold");

    //     // Retreive NFT data from SC storage.
    //     let curNft = self.nft_detail().get(&(nft_token_id.clone(), nft_nonce.clone())).unwrap();

    //     require!(nft_token_id == curNft.token, "Invalid token used as payment");
    //     require!(nft_nonce == curNft.nonce, "Invalid nonce for payment token");
    //     require!(payment_amount == curNft.amount, "Invalid amount as payment");

    //     //___________________
    //     // Calculacte marketplace 3% marketplace_service_fee.
    //     let marketplace_service_fee = curNft.amount.clone() * MARKETPLACE_CUT / PERCENTAGE_TOTAL; 
    //     // Calculacte NFT creator royalties.
    //     let royalties = curNft.amount.clone() * ROYALTIES / PERCENTAGE_TOTAL; 
    //     // Calculate seller's revenue.
    //     let leftovermoney = curNft.amount.clone() - marketplace_service_fee.clone() - royalties.clone();

    //     //___________________
    //     // buyer's address
    //     let caller = self.blockchain().get_caller();
    //     // marketplace smart contract owner
    //     let marketplace_owner = self.blockchain().get_owner_address();
    //     // NFT owner (seller)
    //     let nftOwner = curNft.owner;

    //     //___________________
    //     // fetch info from NFT that is on balance of smart contract now.
    //     let scAddress = self.blockchain().get_sc_address();
    //     let tokenIfo = self.blockchain().get_esdt_token_data(&scAddress, &nft_token_id.clone(), nft_nonce.clone());

    //     //___________________
    //     // send NFT to buyer
    //     self.send().direct(
    //         &caller,
    //         &nft_token_id,
    //         nft_nonce,
    //         &BigUint::from(NFT_AMOUNT),
    //         &[],
    //     );

    //     // send marketplace fee to marketplace owner
    //     self.send().direct(
    //         &marketplace_owner,
    //         &TokenIdentifier::egld(),
    //         0,
    //         &marketplace_service_fee,
    //         &[],
    //     );

    //     // send royalties to NFT creator
    //         self.send().direct(
    //         &tokenIfo.creator,
    //         &TokenIdentifier::egld(),
    //         0,
    //         &royalties,
    //         &[],
    //     );

    //         // send revenue to NFT seller
    //         self.send().direct(
    //         &nftOwner,
    //         &TokenIdentifier::egld(),
    //         0,
    //         &leftovermoney,
    //         &[],
    //     );

    //     // Clear NFT storage data after it's sold.
    //     self.nft_detail().remove(&(nft_token_id.clone(), nft_nonce.clone()));

    //     SCResult::Ok(())
    // }

    // #[payable("*")] 
    // #[endpoint(update_price)]
    // fn update_price(
    //     &self,
    //     token_id: TokenIdentifier,
    //     nonce: u64,
    //     payment_amount: BigUint,    
    // ) -> SCResult<()> {
    //     let caller = self.blockchain().get_caller();
    //     // Retreive NFT data from SC storage.
    //     let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();

    //     require!(caller == curNft.owner, "You are not the owner of this token");
    //     require!(token_id == curNft.token, "Invalid token");
    //     require!(nonce == curNft.nonce, "Invalid nonce");

    //     self.nft_detail().insert((token_id.clone(), nonce.clone()), NftListing{
    //         owner: curNft.owner,
    //         token: curNft.token,
    //         nonce: curNft.nonce,
    //         amount: payment_amount,
    //     });

    //     SCResult::Ok(())
    // }


    // #[payable("*")] 
    // #[endpoint(cancel_listing)]
    // fn cancel_listing(
    //     &self,
    //     token_id: TokenIdentifier,
    //     nonce: u64,
    // ) -> SCResult<()> {
    //     // Check the mapped value exists.
    //     require!(
    //         self.nft_detail().contains_key(&(token_id.clone(), nonce.clone())),
    //         "Invalid NFT token, nonce or NFT was already sold");

    //     let caller = self.blockchain().get_caller();
    //     // Retreive NFT data from SC storage.
    //     let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();

    //     require!(caller == curNft.owner, "You are not the owner of this token");
    //     require!(token_id == curNft.token, "Invalid token");
    //     require!(nonce == curNft.nonce, "Invalid nonce");
        
    //     self.nft_detail().remove(&(token_id.clone(), nonce.clone()));

    //     self.send().direct(
    //         &caller,
    //         &token_id,
    //         nonce,
    //         &BigUint::from(NFT_AMOUNT),
    //         &[],
    //     );

    //     SCResult::Ok(())
    // }

    // Map NFT using some custom mapper identifier.
    // Map NFT by unique token+nonce pair for NFT object.   
    // #[storage_mapper("NftListing")]
    // fn nft_detail(&self) -> MapMapper<(TokenIdentifier, u64), NftListing<Self::Api>>;

    // #[allow(clippy::type_complexity)]
    // #[view(get_listing)]
    // fn get_listing(
    //     &self,
    //     token_id: TokenIdentifier,
    //     nonce: u64,
    // ) -> OptionalValue<MultiValue4<ManagedAddress, TokenIdentifier, u64, BigUint>> {
    //     if !self.nft_detail().contains_key(&(token_id.clone(), nonce.clone())) {
    //         // NFT was already sold
    //         OptionalValue::None
    //     } else {
    //         // Retreive NFT data from SC storage.
    //         let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();
    //         OptionalValue::Some((
    //             curNft.owner,
    //             curNft.token,
    //             curNft.nonce,
    //             curNft.amount,
    //             ).into())
    //     }
    // }

    /* ________________ */
    /* Helper functions */

    // Helper function to get NFT info from the chain.
    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }
}
