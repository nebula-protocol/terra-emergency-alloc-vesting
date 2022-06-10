# Terra 2.0 Emergency LUNA Allocation Vesting Contract

Smart contract for storing, approving, and claiming LUNA allocated to ecosystem projects as of [Prop 446](https://agora.terra.money/discussion/5332-vote-in-progress-prop-446-proposed-distribution-method-for-05-emergency-luna-allocation-version-3).

## Messages

### InstantiateMsg

```json
{
    /// Master address who can update tollgate / status of all vestings
    master_address: Option<String>,
    /// Specific vesting denom
    denom: String,
    /// A list of vestings
    vestings: Vec<Vesting>,
}
```

### ExecuteMsgs

Approve the next tollgate for `recipient`. A tollgate can be approved as long as `current_timestamp` is greater than the time when the tollgate is hit.

This message can only be called by the `master_address` account.

#### ApproveTollgate

```json
/// ApproveTollgate either increments tollgate or deactivate a vesting.
approve_tollgate {
    /// Recipient address of a protocol
    recipient: String,
    /// New vesting status
    approve: bool,
}
```

#### Claim

Claim all unlocked and eligible LUNA.

```json
/// Claim unlocked vesting
claim {

}
```

### QueryMsgs

#### VestingInfo

##### Request

Query the vesting information for a recipient

```json
/// VestingInfo returns the vesting information of the specified recipient
vesting_info {
    /// Recipient address of a protocol
    recipient: String,
}
```

##### Response

```json
vesting_info {
    /// Recipient address of a protocol
    pub recipient: Addr,
    /// Vesting valid status
    pub active: bool,
    /// Current approved pollgates, in periods
    pub approved_periods: u64,
    /// Total vesting periods
    pub total_periods: u64,
    /// Previously claimed period, start at 0
    pub last_claimed_period: u64,
    /// Total vesting amount
    pub total_amount: Uint128,
    /// Claimed vesting amount
    pub claimed_amount: Uint128,
    /// Unclaimed amount
    pub vested_amount: Uint128,
    /// Claimable amount for each period
    pub amount_per_period: Uint128,
}
```