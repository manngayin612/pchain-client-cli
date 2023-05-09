/*
    Copyright © 2023, ParallelChain Lab 
    Licensed under the Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
*/

//! Methods related to subcommand `query` in `pchain-client`.

use std::collections::{HashSet, HashMap};
use std::path::PathBuf;
use pchain_client_rs::{Client, base64url_to_bytes32};
use pchain_types::rpc::*;

use crate::command::{Query, Validators};
use crate::result::{ClientResponse, display_beautified_rpc_result};
use crate::display_msg::DisplayMsg;
use crate::display_types::CallArgument;
use crate::config::Config;
use crate::utils::read_file_to_utf8string;

// `match_query_subcommand` matches a CLI argument to its corresponding `Query` subcommand and processes 
//  the request.
//  # Arguments
//  * `query_subcommand` - query subcommand from CLI
//  * `config` - networking config for Client
//  
pub async fn match_query_subcommand(query_subcommand: Query, config: Config) {
    let url = config.get_url();
    let pchain_client = Client::new(url);

    match query_subcommand {
        Query::Balance{address} => {
            let sender_address: pchain_types::PublicAddress = match base64url_to_bytes32(&address) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("sender"), address, e));
                    std::process::exit(1);
                }
            };
            
            let response = pchain_client
                            .state(&StateRequest {
                                accounts: HashSet::from([sender_address]),
                                include_contract: false,
                                storage_keys: HashMap::from([])
                            })
                            .await;

            display_beautified_rpc_result(ClientResponse::Balance(response));
        },
        Query::Nonce{address} => {
            let sender_address: pchain_types::PublicAddress = match base64url_to_bytes32(&address) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("sender"), address, e));
                    std::process::exit(1);
                }
            };

            let response = pchain_client
                            .state(&StateRequest {
                                accounts: HashSet::from([sender_address]),
                                include_contract: false,
                                storage_keys: HashMap::from([])
                            })
                            .await;

            display_beautified_rpc_result(ClientResponse::Nonce(response));
        },
        Query::Contract{address, destination} => {
            let contract_address: pchain_types::PublicAddress = match base64url_to_bytes32(&address) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("contract"), address, e));
                    std::process::exit(1);
                }
            };     

            let response = pchain_client
                            .state(&StateRequest {
                                accounts: HashSet::from([contract_address]),
                                include_contract: true,
                                storage_keys: HashMap::from([])
                            })
                            .await;

            display_beautified_rpc_result(ClientResponse::Contract(response, destination));
        },
        Query::Block { block_height, ref block_hash, ref tx_hash, latest }
            | Query::BlockHeader { block_height, ref block_hash, ref tx_hash, latest }  => {
                
            let mut count_args = 0;
            if block_height.is_some() { count_args += 1; }
            if block_hash.is_some() { count_args += 1; }
            if tx_hash.is_some() { count_args += 1; }
            if latest { count_args += 1; }
            if count_args != 1{
                println!("{}", DisplayMsg::IncorrectCombinationOfIdentifiers(String::from("latest, block-num, block-hash, tx-hash")));
                std::process::exit(1)
            }

            if latest{
                let response = pchain_client
                    .highest_committed_block()
                    .await;

                let block_hash = match response {
                    Ok(HighestCommittedBlockResponse{block_hash: Some(block_hash)}) => block_hash,
                    Err(e) => {
                        println!("{}", DisplayMsg::RespnoseWithHTTPError(e));
                        std::process::exit(1);
                    },
                    _ => {
                        println!("{}", DisplayMsg::CannotFindLatestBlock);
                        std::process::exit(1);                               
                    }
                };
                
                match query_subcommand {
                    Query::BlockHeader { block_height: _, block_hash: _, tx_hash: _, latest: _ } => {
                        let response = pchain_client
                        .block_header(&BlockHeaderRequest{ block_hash })
                        .await;
                        
                        display_beautified_rpc_result(ClientResponse::BlockHeader(response));
                    },
                    _ => {
                        let response = pchain_client
                            .block(&BlockRequest{ block_hash })
                            .await;

                        display_beautified_rpc_result(ClientResponse::Block(response));
                    },
                };

            } else if let Some(block_height) = block_height {
                let response = pchain_client
                    .block_hash_by_height(&BlockHashByHeightRequest{block_height})
                    .await;

                let block_hash = match response {
                    Ok(BlockHashByHeightResponse{block_height: _, block_hash: Some(block_hash)}) => block_hash,
                    Err(e) => {
                        println!("{}", DisplayMsg::RespnoseWithHTTPError(e));
                        std::process::exit(1);
                    },
                    _ => {
                        println!("{}", DisplayMsg::CannotFindReleventBlock);
                        std::process::exit(1);              
                                    
                    }
                };
                
                match query_subcommand {
                    Query::BlockHeader { block_height: _, block_hash: _, tx_hash: _, latest: _ } => {
                        let response = pchain_client
                        .block_header(&BlockHeaderRequest{ block_hash })
                        .await;
                        
                        display_beautified_rpc_result(ClientResponse::BlockHeader(response));
                    },
                    _ => {
                        let response = pchain_client
                            .block(&BlockRequest{ block_hash })
                            .await;

                        display_beautified_rpc_result(ClientResponse::Block(response));
                    },
                };

            } else if let Some(hash) = block_hash {
                let block_hash: pchain_types::Sha256Hash = match base64url_to_bytes32(&hash) {
                    Ok(hash) => hash,
                    Err(e) => {
                        println!("{}", DisplayMsg::FailToDecodeBase64Hash(String::from("block"), String::from(hash), e));
                        std::process::exit(1);
                    }
                };

                match query_subcommand {
                    Query::BlockHeader { block_height: _, block_hash: _, tx_hash: _, latest: _ } => {
                        let response = pchain_client
                        .block_header(&BlockHeaderRequest{ block_hash })
                        .await;
                        
                        display_beautified_rpc_result(ClientResponse::BlockHeader(response));
                    },
                    _ => {
                        let response = pchain_client
                            .block(&BlockRequest{ block_hash })
                            .await;

                        display_beautified_rpc_result(ClientResponse::Block(response));
                    },
                }
            } else if let Some(hash) = tx_hash {
                let transaction_hash: pchain_types::Sha256Hash = match base64url_to_bytes32(&hash) {
                    Ok(hash) => hash,
                    Err(e) => {
                        println!("{}", DisplayMsg::FailToDecodeBase64Hash(String::from("transaction"), String::from(hash), e));
                        std::process::exit(1);
                    }
                };

                let response = pchain_client
                    .transaction_position(&TransactionPositionRequest{transaction_hash})
                    .await;

                let block_hash = match response {
                    Ok(TransactionPositionResponse{transaction_hash: _, block_hash: Some(block_hash), position: _}) => block_hash,
                    Err(e) => {
                        println!("{}", DisplayMsg::RespnoseWithHTTPError(e));
                        std::process::exit(1);
                    },
                    _ => {
                        println!("{}", DisplayMsg::CannotFindReleventBlock);
                        std::process::exit(1);              
                                    
                    }
                };
                
                match query_subcommand {
                    Query::BlockHeader { block_height: _, block_hash: _, tx_hash: _, latest: _ } => {
                        let response = pchain_client
                        .block_header(&BlockHeaderRequest{ block_hash })
                        .await;
                        
                        display_beautified_rpc_result(ClientResponse::BlockHeader(response));
                    },
                    _ => {
                        let response = pchain_client
                            .block(&BlockRequest{ block_hash })
                            .await;

                        display_beautified_rpc_result(ClientResponse::Block(response));
                    },
                }
            }
        },
        Query::Tx { tx_hash } => {        
            let tx_hash: pchain_types::Sha256Hash = match base64url_to_bytes32(&tx_hash) {
                Ok(hash) => hash,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Hash(String::from("transaction"), tx_hash, e));
                    std::process::exit(1);
                }
            };

            let response = pchain_client
                .transaction(&TransactionRequest{ transaction_hash: tx_hash, include_receipt: true })
                .await;
                
            display_beautified_rpc_result(ClientResponse::Transaction(response));
                         
        },
        Query::Receipt {tx_hash} => {
            let tx_hash: pchain_types::Sha256Hash = match base64url_to_bytes32(&tx_hash) {
                Ok(hash) => hash,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Hash(String::from("transaction"), tx_hash, e));
                    std::process::exit(1);
                }
            };
            
            let response = pchain_client
                .receipt(&ReceiptRequest{ transaction_hash: tx_hash })
                .await;

            display_beautified_rpc_result(ClientResponse::Receipt(response));
        },
        Query::Storage { address, key } => {
            let contract_address: pchain_types::PublicAddress = match base64url_to_bytes32(&address) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("contract"), address, e));
                    std::process::exit(1);
                }
            };
            let world_state_key: Vec<u8> = match pchain_types::Base64URL::decode(&key) {
                Ok(k) => k,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64String(String::from("world state key"), key, e.to_string()));
                    std::process::exit(1);
                }
            };            
            
            let response = pchain_client
                .state(&StateRequest{ 
                    accounts: HashSet::from([]), 
                    include_contract: true, 
                    storage_keys: HashMap::from([(contract_address, HashSet::from([world_state_key]))])
                })
                .await;

            display_beautified_rpc_result(ClientResponse::State(response));
        },
        Query::View { target, method, arguments} => {
            let contract_address: pchain_types::PublicAddress = match base64url_to_bytes32(&target) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("target"), target, e));
                    std::process::exit(1);
                }
            };
        
            let args = match arguments {
                Some(path) => {
                    let path_to_json = PathBuf::from(&path);
                    let arguments_json = match read_file_to_utf8string(path_to_json.clone()) {
                        Ok(result) => result,
                        Err(e) => {
                            println!("{}", DisplayMsg::FailToOpenOrReadFile(String::from("view argment json"), path_to_json, e));
                            std::process::exit(1);
                        }
                    };

                    let call = match pchain_client_rs::CallArguments::from_json(&arguments_json) {
                        Ok(result) => result,
                        Err(e) => {
                            println!("{}", DisplayMsg::FailToDecodeJson(String::from("call argument"), path_to_json, e));
                            std::process::exit(1);
                        }
                    };

                    if call.arguments.len() == 0 { 
                        None 
                    } else { 
                       let mut call_arguments = Vec::new();
                       for argument in call.arguments {
                            call_arguments.push(
                                CallArgument{ argument_type: argument.0, argument_value: argument.1  }
                            );
                       }
        
                       Some(call_arguments)
                    }
                },
                None => None
            };
            
            let response = pchain_client
                .view(&ViewRequest{ 
                        target: contract_address,
                        method: method.into_bytes(),
                        arguments: CallArgument::serialize_arguments(args)
                })                        
                .await;

            display_beautified_rpc_result(ClientResponse::View(response));
        },
        Query::Validators { validator_subcommand } => {
            match validator_subcommand {
                Validators::Previous { with_delegator } => {
                    let response = pchain_client
                        .validator_sets(&ValidatorSetsRequest{ 
                            include_prev: true, 
                            include_prev_delegators: with_delegator , 
                            include_curr: false, 
                            include_curr_delegators: false, 
                            include_next: false, 
                            include_next_delegators: false 
                        })
                        .await;
    
                    display_beautified_rpc_result(ClientResponse::PreviousValidatorSet(response));
                },
                Validators::Current { with_delegator }=> {
                    let response = pchain_client
                    .validator_sets(&ValidatorSetsRequest{ 
                        include_prev: false, 
                        include_prev_delegators: false,
                        include_curr: true, 
                        include_curr_delegators: with_delegator,
                        include_next: false, 
                        include_next_delegators: false 
                    })                        
                    .await;

                    display_beautified_rpc_result(ClientResponse::CurrentValidatorSet(response));
                },
                Validators::Next{ with_delegator } => {
                    let response = pchain_client
                    .validator_sets(&ValidatorSetsRequest{ 
                        include_prev: false, 
                        include_prev_delegators: false , 
                        include_curr: false, 
                        include_curr_delegators: false, 
                        include_next: true, 
                        include_next_delegators: with_delegator 
                    })                        
                    .await;

                    display_beautified_rpc_result(ClientResponse::NextValidatorSet(response));
                },
            }
        },
        Query::Deposit { operator, owner } => {
            let operator: pchain_types::PublicAddress = match base64url_to_bytes32(&operator) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("operator"), operator, e));
                    std::process::exit(1);
                }
            };

            let owner: pchain_types::PublicAddress = match base64url_to_bytes32(&owner) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("owner"), owner, e));
                    std::process::exit(1);
                }
            };

           let response = pchain_client
                .deposits(&DepositsRequest{stakes: HashSet::from([(operator, owner)])})
                .await;

            display_beautified_rpc_result(ClientResponse::Deposit(response))
        },
        Query::Pool { operator, with_stakes} => {
            let operator: pchain_types::PublicAddress = match base64url_to_bytes32(&operator) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("operator"), operator, e));
                    std::process::exit(1);
                }
            };

           let response = pchain_client
                .pools(&PoolsRequest{operators: HashSet::from([operator]), include_stakes: with_stakes})
                .await;


            display_beautified_rpc_result(ClientResponse::Pool(response))
        },
        Query::Stake { operator, owner } => {
            let operator: pchain_types::PublicAddress = match base64url_to_bytes32(&operator) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("operator"), operator, e));
                    std::process::exit(1);
                },
            };

            let owner: pchain_types::PublicAddress = match base64url_to_bytes32(&owner) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("{}", DisplayMsg::FailToDecodeBase64Address(String::from("owner"), owner, e));
                    std::process::exit(1);
                }
            };
            
            let response = pchain_client
                        .stakes(&StakesRequest{stakes:  HashSet::from([(operator, owner)])})
                        .await;
            display_beautified_rpc_result(ClientResponse::StakePower(response))
        }
    }
}