use std::result::Result::Ok;
use crate::utils::*;
use anyhow::{ensure, Result};
use cp_amm::{
    params::swap::TradeDirection,
    state::{fee::FeeMode, Pool, SwapResult2},
};
use cp_amm::state::SimulateSwapResult2;

pub fn get_quote(
    pool: &Pool,
    current_timestamp: u64,
    current_slot: u64,
    actual_amount_in: u64,
    a_to_b: bool,
    has_referral: bool,
) -> Result<SwapResult2> {
    ensure!(actual_amount_in > 0, "amount is zero");

    let current_point = get_current_point(pool.activation_type, current_slot, current_timestamp)?;

    ensure!(is_swap_enable(pool, current_point)?, "Swap is disabled");

    let trade_direction = if a_to_b {
        TradeDirection::AtoB
    } else {
        TradeDirection::BtoA
    };

    let fee_mode = &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, has_referral)?;

    Ok(pool.get_swap_result_from_exact_input(
        actual_amount_in,
        fee_mode,
        trade_direction,
        current_point,
    )?)
}

pub fn get_quote_simulate(
    old_pool: &Pool,
    current_timestamp: u64,
    current_slot: u64,
    actual_amount_in: u64,
    a_to_b: bool,
    has_referral: bool,
) -> Result<SimulateSwapResult2> {
    ensure!(actual_amount_in > 0, "amount is zero");
    let mut pool = old_pool.clone();
    let current_point = get_current_point(pool.activation_type, current_slot, current_timestamp)?;

    ensure!(is_swap_enable(&pool, current_point)?, "Swap is disabled");

    let trade_direction = if a_to_b {
        TradeDirection::AtoB
    } else {
        TradeDirection::BtoA
    };

    let fee_mode = &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, has_referral)?;
    match pool.get_swap_result_from_exact_input(
        actual_amount_in,
        fee_mode,
        trade_direction,
        current_point,
    ) {
        Ok(quote) => {
            pool.apply_swap_result(&quote,fee_mode,current_timestamp);
            Ok(SimulateSwapResult2{
                swap_result2: quote,
                new_pool: pool,
                is_update: true,
            })
        }
        Err(e) => Ok(SimulateSwapResult2{
            swap_result2: SwapResult2{
                included_fee_input_amount: 0,
                excluded_fee_input_amount: 0,
                amount_left: 0,
                output_amount: 0,
                next_sqrt_price: 0,
                trading_fee: 0,
                protocol_fee: 0,
                partner_fee: 0,
                referral_fee: 0,
            },
            new_pool: pool,
            is_update: false,
        }),
    }
}
