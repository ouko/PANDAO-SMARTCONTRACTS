//! blue print for

use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct PraposalStatus {
    praposal: String,
    yes: Decimal,
    no: Decimal,
}

#[blueprint]
mod radixdao {
    enable_method_auth! {


     methods{
        change_praposal => restrict_to : [OWNER] ;
        get_praposal_status => PUBLIC ;
        obtain_token => PUBLIC ;
        withdraw_power => PUBLIC ;
      }
    }

    pub struct TokenWeigtedDao {
        ongoing_praposal: String,
        yes: Decimal,
        no: Decimal,
        // shares represents the total amount of the DAO
        shares: Vault,
        voting_power_tokens: Vault,
        // price represents the price pf token
        power_price: Decimal,
        buy_back_price: Decimal,
        praposal_active: bool,
       
    }

    impl TokenWeigtedDao {
        /// function arguments
        /// organization_name => represents the name of the organization which is building this dao
        /// tottal supply => number of tokens which will be present
        /// divisibility => how tokens are divided
        /// price => price of the token/voting power
        /// buy_back_price => this is the price that user can have when with drawing power
        pub fn initiate(
            organization_name: String,
            token_supply: i32,
            divisibility: u8,
            // define the price of 1 token for xrd
            power_price: Decimal,
            buy_back_price: Decimal,
            org_ico_url:String,
            power_token_url:String
        ) -> (Global<TokenWeigtedDao>,Bucket) {
            // reserve an address for the component
            let (address_reservation, _) =
                Runtime::allocate_component_address(TokenWeigtedDao::blueprint_id());

            let owner_badge_description = format!("{}'s owner badge", &organization_name);
            // create a owner role, this role is only for changing the praposal and inserting a new praposal
            let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .metadata(metadata!(
                    init{
                        "name"=>owner_badge_description,locked;
                        "icon_url" => Url::of(&org_ico_url), locked;
                    }
                ))
                .mint_initial_supply(1)
                .into();

            // create nft to be sold for voting purpose
            let voting_power_tokens: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(divisibility)
                .metadata(metadata!(init{
                    "name"=>"voting power dao",locked ;
                    // set the name of the organization
                    "organization name" => organization_name , locked ;
                    "icon_url" => Url::of(&power_token_url), locked;
                }))
                .mint_initial_supply(token_supply)
                .into();

            let component = Self {
                power_price,
                buy_back_price,
                ongoing_praposal: "no active praposal".into(),
                yes: 0.into(),
                no: 0.into(),
                shares: Vault::new(XRD),
                voting_power_tokens: Vault::with_bucket(voting_power_tokens),
                praposal_active: false,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                owner_badge.resource_address()
            ))))
            .with_address(address_reservation)
            .globalize();

            (component,owner_badge)
        }

        pub fn obtain_token(&mut self, mut xrd: Bucket, token_amount: Decimal) -> (Bucket,Bucket) {
            // check if the supplied amount is greater or equal to the token amount required to buy the token
            assert!((self.power_price * token_amount) <= xrd.amount());
            let collected_shares = xrd.take(self.power_price * token_amount);
            let power_share = self.voting_power_tokens.take(token_amount) ;
            self.shares.put(collected_shares) ;
            (xrd,power_share)
        }

        pub fn withdraw_power(&mut self, voting_power: Bucket) -> Bucket {
             // put the voting power back
             let power_amount = voting_power.amount() ;
             self.voting_power_tokens.put(voting_power) ;
             self.shares.take(power_amount * self.buy_back_price )
        }

        // function to check status of ongoing praposal
        pub fn get_praposal_status(&self) -> PraposalStatus {
            PraposalStatus {
                praposal: self.ongoing_praposal.clone(),
                yes: self.yes,
                no: self.no,
            }
        }
        pub fn change_praposal(&mut self,new_praposal:String) {
                 self.ongoing_praposal = new_praposal ;
        }
    }
}

