use alloy::sol;

sol!(
    #[sol(rpc)]
    IERC20,
    "abi/ERC20.json"
);

sol!(
    #[sol(rpc)]
    IERC20Fallback,
    "abi/ERC20Fallback.json"
);
