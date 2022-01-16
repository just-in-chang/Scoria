#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use cosmwasm_std::Addr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg, ScoreResponse};
use crate::state::{State, SCORES, STATE};

const CONTRACT_NAME: &str = "scoria";
const CONTRACT_VERSION: &str = "1";

/// Instantiation function for the smart contract and saves the creator's address as the owner in the contract's state.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // Sets the owner of the state
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Execution function for updating an addresses's score.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateScore { address, score } => try_update_score(deps, info, address, score),
    }
}

/// Attempts to update the score of an address.
pub fn try_update_score(
    deps: DepsMut,
    info: MessageInfo,
    addr: Addr,
    score: i32,
) -> Result<Response, ContractError> {
    // Throws an unauthorized contract error if the sender is not the owner
    if info.sender != STATE.load(deps.storage).unwrap().owner {
        return Err(ContractError::Unauthorized {});
    }

    // Updates the score of an address
    SCORES.update(deps.storage, &addr, |x| -> StdResult<_> {
        match x {
            Some(_) => Ok(score),
            None => Ok(score),
        }
    })?;

    Ok(Response::new().add_attribute("method", "try_update_score"))
}

/// Query function to obtain the score of an address
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<ScoreResponse, ContractError> {
    match msg {
        QueryMsg::GetScore { address } => query_score(deps, address),
    }
}

/// Determines whether or not a score exists for an address and returns its score
fn query_score(deps: Deps, addr: Addr) -> Result<ScoreResponse, ContractError> {
    if SCORES.has(deps.storage, &addr) {
        return Ok(ScoreResponse {
            score: SCORES.load(deps.storage, &addr).unwrap(),
        });
    }

    return Err(ContractError::AddressNotFound {});
}

/// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};

    /// Tests proper initialization of smart contract
    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let info = mock_info("creator", &coins(1000, "token"));
        let res = instantiate(deps.as_mut(), mock_env(), info).unwrap();
        assert_eq!(0, res.messages.len());
    }

    /// Tests a successful updating of the score of an address
    #[test]
    fn update_and_query() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone()).unwrap();

        // Assigns the owner's address a score of 3
        let msg = ExecuteMsg::UpdateScore {
            address: info.clone().sender,
            score: 3,
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Queries the score of the owner's address
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetScore {
                address: info.clone().sender,
            },
        )
        .unwrap();
        assert_eq!(3, res.score);
    }

    /// Tests an unsuccessful updating of the score of an address
    #[test]
    fn fail_to_update() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let info = mock_info("creator", &coins(2, "token"));
        let bad_guy = mock_info("crook", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone()).unwrap();

        // Attempts (fails) to assign the bad guy's address a score of 9999999
        let msg = ExecuteMsg::UpdateScore {
            address: bad_guy.clone().sender,
            score: 9999999,
        };
        let err_res = execute(deps.as_mut(), mock_env(), bad_guy.clone(), msg);
        match err_res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    /// Tests an unsuccessful query of the score of an address
    #[test]
    fn fail_to_find() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone()).unwrap();

        // Queries the (nonexistent) score of the owner
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetScore {
                address: info.clone().sender,
            },
        );
        match res {
            Err(ContractError::AddressNotFound {}) => {}
            _ => panic!("Must return address not found error"),
        }
    }
}
