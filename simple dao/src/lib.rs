use scrypto::prelude::*;

#[blueprint]
mod global_dao {

    enable_method_auth! {
        // decide which methods are public and which are restricted to certain roles
        roles {
            voter => updatable_by: [OWNER];
        },
        methods {
           vote => restrict_to : [voter] ;
        }
    }

    struct Mydao {
        yes: i32,
        no: i32,
        voting_right: Vault,
        praposal: String,
    }

    impl Mydao {
        fn initiate(praposal: String) -> (Global<Mydao>, Bucket) {
            let (address_reservation, _) =
                Runtime::allocate_component_address(Mydao::blueprint_id());

            // define an owner badge
            // owner badge holder can give voting right to other people
            let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init{
                        "name" => "admin/owner badge",locked;
                    }
                ))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

            // define a voting right badge
            let voting_right_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init{
                        "name"=> "voting right badge",locked ;
                    }
                ))
                .divisibility(1)
                .mint_initial_supply(10)
                .into();

            let voter_badge_address = voting_right_badge.resource_address();

            let component = Self {
                yes: 0,
                no: 0,
                voting_right: Vault::with_bucket(voting_right_badge),
                praposal,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                owner_badge.resource_address()
            ))))
            .roles(roles! {
                    voter => rule!( require( voter_badge_address )) ;
            })
            .with_address(address_reservation)
            .globalize();

            (component, owner_badge)
        }

        pub fn vote(&self) {}
    }
}
