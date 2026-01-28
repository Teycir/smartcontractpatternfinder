// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract VulnerableGovernance {
    // SHOULD MATCH: No timelock check
    function execute(uint256 proposalId) external {
        // Execute without timelock
        _executeProposal(proposalId);
    }
    
    function _executeProposal(uint256 id) internal {}
}

contract SafeGovernance {
    uint256 public constant TIMELOCK_DELAY = 2 days;
    mapping(uint256 => uint256) public proposalEta;
    
    // SHOULD NOT MATCH: Has timelock check
    function execute(uint256 proposalId) external {
        require(block.timestamp >= proposalEta[proposalId], "Timelock not expired");
        require(proposalEta[proposalId] != 0, "Proposal not queued");
        _executeProposal(proposalId);
    }
    
    function _executeProposal(uint256 id) internal {}
}

contract UnrelatedContract {
    // SHOULD NOT MATCH: Not governance related
    function execute() external {
        // Just a regular execute function
    }
}
