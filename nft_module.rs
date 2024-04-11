elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const NFT_AMOUNT: u32 = 1;
const MARKETPLACE_CUT: u32 = 300;
const ROYALTIES: u32 = 1000;
const PERCENTAGE_TOTAL: u64 = 10000;

#[derive(TypeAbi, TopEncode, TopDecode, ManagedVecItem, NestedEncode, NestedDecode)]
pub struct NftDetail<M: ManagedTypeApi> {
    pub token: TokenIdentifier<M>,
    pub nonce: u64,
    pub owner: ManagedAddress<M>,
    pub amount: BigUint<M>,
}

#[elrond_wasm::module]
pub trait NftModule {
    // owner-only endpoints
    #[payable("*")] 
    #[endpoint(list_nft)] // endpoint name
    fn list_nft( // list NFT e.g. TR11-531aff-01 for sale
        &self,
        token_id: TokenIdentifier, // collection identifier, e.g. name + 6 random symbols, e.g. TR11-531aff(-01)
        nonce: u64, // nonce, e.g. id for NFT in collection, e.g. (TR11-531aff)-01
        selling_price: BigUint, // e.g. 0.5 EGLD
        owner: ManagedAddress, // seller's address to pay margin
    ) -> SCResult<()> {
        // TODO 
        // Call for NFT data as self.nft_detal
        // as let tokenIfo = self.blockchain().get_esdt_token_data(&scAddress, &nft_token_id.clone(), nft_nonce.clone());
        // to validate Elrond SC really has this NFT in transaction runtime, that will make endpoint vulnerable for this NFT to other transactions, only in time NFT is on Elrond wallet
        // probably as it's not all committed yet, the NFT is on Elrond's wallet only during the tx execution time and API will take in into account, so this will work as from this state of blockchain
        // that will eliminate any attempt of fraud forever.

        // Write last owner of NFT into storage.
        // Set SC metadata for mapped NFT in storage.
        // Use (TR11-531aff) (01) as for mapping, must be always unique
        self.nft_detail().insert((token_id.clone(), nonce.clone()), NftDetail{
            owner: owner,
            token: token_id,
            nonce: nonce,
            amount: selling_price,
        });

        Ok(())
    }

    // endpoints
    #[payable("EGLD")]
    #[allow(clippy::too_many_arguments)]
    #[endpoint(buy_nft)]
    fn buy_nft(
        &self,
        #[payment_amount] payment_amount: BigUint,
        nft_token_id: TokenIdentifier,
        nft_nonce: u64,
    ) -> SCResult<()> {

        // Check the mapped value exists.
        require!(
            self.nft_detail().contains_key(&(nft_token_id.clone(), nft_nonce.clone())),
            "Invalid NFT token, nonce or NFT was already sold");

        // Retreive NFT data from SC storage.
        let curNft = self.nft_detail().get(&(nft_token_id.clone(), nft_nonce.clone())).unwrap();

        require!(nft_token_id == curNft.token, "Invalid token used as payment");
        require!(nft_nonce == curNft.nonce, "Invalid nonce for payment token");
        require!(payment_amount == curNft.amount, "Invalid amount as payment");

        //___________________
        // Calculacte marketplace 3% marketplace_service_fee.
        let marketplace_service_fee = curNft.amount.clone() * MARKETPLACE_CUT / PERCENTAGE_TOTAL; 
        // Calculacte NFT creator royalties.
        let royalties = curNft.amount.clone() * ROYALTIES / PERCENTAGE_TOTAL; 
        // Calculate seller's revenue.
        let leftovermoney = curNft.amount.clone() - marketplace_service_fee.clone() - royalties.clone();

        //___________________
        // buyer's address
        let caller = self.blockchain().get_caller();
        // marketplace smart contract owner
        let marketplace_owner = self.blockchain().get_owner_address();
        // NFT owner (seller)
        let nftOwner = curNft.owner;

        //___________________
        // fetch info from NFT that is on balance of smart contract now.
        let scAddress = self.blockchain().get_sc_address();
        let tokenIfo = self.blockchain().get_esdt_token_data(&scAddress, &nft_token_id.clone(), nft_nonce.clone());

        //___________________
        // send NFT to buyer
        self.send().direct(
            &caller,
            &nft_token_id,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
            &[],
        );

        // send marketplace fee to marketplace owner
        self.send().direct(
            &marketplace_owner,
            &TokenIdentifier::egld(),
            0,
            &marketplace_service_fee,
            &[],
        );

        // send royalties to NFT creator
            self.send().direct(
            &tokenIfo.creator,
            &TokenIdentifier::egld(),
            0,
            &royalties,
            &[],
        );

            // send revenue to NFT seller
            self.send().direct(
            &nftOwner,
            &TokenIdentifier::egld(),
            0,
            &leftovermoney,
            &[],
        );

        // Clear NFT storage data after it's sold.
        self.nft_detail().remove(&(nft_token_id.clone(), nft_nonce.clone()));

        Ok(())
    }

    #[payable("*")] 
    #[endpoint(update_price)]
    fn update_price(
        &self,
        token_id: TokenIdentifier,
        nonce: u64,
        payment_amount: BigUint,    
    ) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        // Retreive NFT data from SC storage.
        let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();

        require!(caller == curNft.owner, "You are not the owner of this token");
        require!(token_id == curNft.token, "Invalid token");
        require!(nonce == curNft.nonce, "Invalid nonce");

        self.nft_detail().insert((token_id.clone(), nonce.clone()), NftDetail{
            owner: curNft.owner,
            token: curNft.token,
            nonce: curNft.nonce,
            amount: payment_amount,
        });

        Ok(())
    }

    #[payable("*")] 
    #[endpoint(cancel_listing)]
    fn cancel_listing(
        &self,
        token_id: TokenIdentifier,
        nonce: u64,
    ) -> SCResult<()> {
        // Check the mapped value exists.
        require!(
            self.nft_detail().contains_key(&(token_id.clone(), nonce.clone())),
            "Invalid NFT token, nonce or NFT was already sold");

        let caller = self.blockchain().get_caller();
        // Retreive NFT data from SC storage.
        let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();

        require!(caller == curNft.owner, "You are not the owner of this token");
        require!(token_id == curNft.token, "Invalid token");
        require!(nonce == curNft.nonce, "Invalid nonce");
        
        self.nft_detail().remove(&(token_id.clone(), nonce.clone()));

        self.send().direct(
            &caller,
            &token_id,
            nonce,
            &BigUint::from(NFT_AMOUNT),
            &[],
        );

        Ok(())
    }

    // Map NFT using some custom mapper identifier.
    // Map NFT by unique token+nonce pair for NFT object.   
    #[storage_mapper("nftDetail")]
    fn nft_detail(&self) -> MapMapper<(TokenIdentifier, u64), NftDetail<Self::Api>>;

    #[allow(clippy::type_complexity)]
    #[view(get_listing)]
    fn get_listing(
        &self,
        token_id: TokenIdentifier,
        nonce: u64,
    ) -> OptionalValue<MultiValue4<ManagedAddress, TokenIdentifier, u64, BigUint>> {
        if !self.nft_detail().contains_key(&(token_id.clone(), nonce.clone())) {
            // NFT was already sold
            OptionalValue::None
        } else {
            // Retreive NFT data from SC storage.
            let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();
            OptionalValue::Some((
                curNft.owner,
                curNft.token,
                curNft.nonce,
                curNft.amount,
                ).into())
        }
    }

    #[allow(clippy::type_complexity)]
    #[view(get_all_listings)]
    fn get_all_listings(&self) 
    -> MultiValueManagedVec<Self::Api, NftDetail<Self::Api>> {
        let storageDetails = self.nft_detail(); // store in a intermediate variable not dropping the results
        let storageIterator = storageDetails.values(); // get iterator from the results retrieved above 
        let mut listingsFound: MultiValueManagedVec<Self::Api, NftDetail<Self::Api>> = MultiValueManagedVec::new();

            for nft in storageIterator {
                listingsFound.push(
                    NftDetail{
                    token: nft.token,
                    nonce: nft.nonce,
                    owner: nft.owner,
                    amount: nft.amount,
                    }
                )
            }

            return listingsFound;
    }
}