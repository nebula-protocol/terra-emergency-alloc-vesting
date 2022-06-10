# Terra 2.0 Emergency LUNA Allocation Vesting Contract

Smart contract for storing, approving, and claiming LUNA allocated to ecosystem projects as of [Prop 446](https://agora.terra.money/discussion/5332-vote-in-progress-prop-446-proposed-distribution-method-for-05-emergency-luna-allocation-version-3).

## Messages

### InstantiateMsg

```json
{
    master_address: Option<String>,
    denom: String,
    vestings: Vec<Vesting>,
}
```

Variables:
- `master_address`: address who can update tollgate / status of all vestings
- `denom`: Vested token's Cosmos SDK coin denom
- `vestings`: list of vesting parameters

### ExecuteMsgs

#### ApproveTollgate

Approve the next tollgate for `recipient`. A tollgate can be approved as long as `current_timestamp` is greater than the time when the tollgate is hit.

If a tollgate is not approved, the remaining LUNA allocation for the `recipient` project is sent back to the `master_address`.

**Note: this message can only be called by the `master_address` account.**

```json
approve_tollgate {
    recipient: String,
    approve: bool,
}
```

Variables:
- `recipient`: the address of the recipient protocol to approve the tollgate for
- `approve`: whether to approve the tollgate (either `true` or `false`)

#### Claim

Claim all unlocked and eligible LUNA.

```json
claim {

}
```

### QueryMsgs

#### VestingInfo

Query the vesting information for a recipient.

##### Request

```json
vesting_info {
    recipient: String,
}
```

Variables:
- `recipient`: the address of the recipient protocol to approve the tollgate for

##### Response

```json
vesting_info {
    pub recipient: Addr,
    pub active: bool,
    pub approved_periods: u64,
    pub total_periods: u64,
    pub last_claimed_period: u64,
    pub total_amount: Uint128,
    pub claimed_amount: Uint128,
    pub vested_amount: Uint128,
    pub amount_per_period: Uint128,
}
```

Variables:
- `recipients`: the address of the recipient protocol to approve the tollgate for
- `active`: vesting valid status
- `approved_periods`: current approved tollgates, in periods
- `total_periods`: total vesting periods
- `last_claimed_period`: previously claimed period, start at 0
- `total_amount`: total vesting amount
- `claimed_amount`: amount of vested tokens claimed
- `vested_amount`: amount of vested tokens still unclaimed
- `amount_per_period`: claimable amount for each period