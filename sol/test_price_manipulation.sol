// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Test cases for price_manipulation.yaml - Based on 2025 DeFi exploits

interface IERC20 {
    function balanceOf(address) external view returns (uint256);
}

interface IUniswapV2Pair {
    function getReserves() external view returns (uint112, uint112, uint32);
}

contract VulnerablePriceManipulation {
    IERC20 public token0;
    IERC20 public token1;
    IUniswapV2Pair public pair;
    address public oracle;

    // SHOULD MATCH: spot-price-from-reserves
    function getPrice() public view returns (uint256) {
        (uint112 reserve0, uint112 reserve1,) = pair.getReserves();
        return uint256(reserve1) * 1e18 / uint256(reserve0); // Spot price
    }

    // SHOULD MATCH: balanceof-this-pricing
    function getTokenPrice() public view returns (uint256) {
        uint256 balance = token0.balanceOf(address(this));
        uint256 supply = token1.balanceOf(address(this));
        return balance / supply; // Flash loan vulnerable
    }

    // SHOULD MATCH: single-oracle-no-fallback
    function fetchPrice() public view returns (uint256) {
        return IOracle(oracle).getPrice();
    }

    // SHOULD MATCH: chainlink-no-staleness-check
    function getChainlinkPrice() public view returns (uint256) {
        (, int256 price,,,) = IChainlink(oracle).latestRoundData();
        return uint256(price);
        // Missing: require(block.timestamp - updatedAt < 3600)
    }

    // SHOULD MATCH: uniswap-getamountout-no-twap
    function swap(uint256 amountIn) public view returns (uint256) {
        return IRouter(oracle).getAmountOut(amountIn, address(token0), address(token1));
    }

    // SHOULD MATCH: price-from-single-pool
    function calculatePrice() public view returns (uint256) {
        uint256 price = pair.getReserves();
        return price; // Single pool, no validation
    }

    // SHOULD MATCH: sqrt-price-no-bounds
    function getV3Price(uint160 sqrtPriceX96) public pure returns (uint256) {
        return uint256(sqrtPriceX96) * uint256(sqrtPriceX96);
        // No bounds check
    }
}

interface IOracle {
    function getPrice() external view returns (uint256);
}

interface IChainlink {
    function latestRoundData() external view returns (uint80, int256, uint256, uint256, uint80);
}

interface IRouter {
    function getAmountOut(uint256, address, address) external view returns (uint256);
}

contract SafePriceManipulation {
    address public twapOracle;
    address public chainlinkOracle;

    function getPrice() public view returns (uint256) {
        return ITWAPOracle(twapOracle).observe(3600);
    }

    function fetchPrice() public view returns (uint256) {
        uint256 price1 = IOracle(chainlinkOracle).getPrice();
        uint256 price2 = ITWAPOracle(twapOracle).currentCumulativePrice();
        require(price1 * 100 / price2 < 105, "Deviation");
        return (price1 + price2) / 2;
    }

    function getChainlinkPrice() public view returns (uint256) {
        (,int256 price,,uint256 updatedAt,) = IChainlink(chainlinkOracle).latestRoundData();
        require(block.timestamp - updatedAt < 3600, "Stale");
        return uint256(price);
    }
}

interface ITWAPOracle {
    function observe(uint256) external view returns (uint256);
    function currentCumulativePrice() external view returns (uint256);
}
