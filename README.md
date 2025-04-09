<p align="center">
  <a href="https://teachfi.network/" target="blank"><img src="https://teachfi.network/teachfi-logo.svg" width="256" alt="TeachFi Logo" /><br/>Test Launchpad</a>
</p>
<br/>
<br/>
<br/>

# Description

This is a child contract of Platform SC. A separate instance is deployed for each platform subscriber.\
It is basically a capital raising platform for the students entrepreneurial ideas. Users whitelisted in the parent Platform SC can launch tokens for their project ideas, in order to raise funds for further development.
<br/>
<br/>
<br/>
## Endpoints

<br/>

```rust
newLaunchpad(
    description: ManagedBuffer,
    token: TokenIdentifier,
    payment_token: TokenIdentifier,
    price: BigUint, // if payment token is USDC (6 decimals), price should be x_000_000
    min_buy_amount: BigUint, // in tokens
    max_buy_amount: BigUint, // in tokens
    start_time: u64,
    end_time: u64
) -> u64
```
>[!IMPORTANT]
>*Requirements:* state = active, token should not be already launched.

>[!NOTE]
>Creates a new launchpad for the specified `token` with the provided parameters. Returns the ID of the newly created Launchpad object.
<br/>

```rust
addTokens(id: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active, launchpad should not have ended.

>[!NOTE]
>Adds tokens for sale in the launchpad corresponding to `id`.
<br/>

```rust
cancelLaunchpad(id: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = launchpad owner, total_sold = 0.

>[!NOTE]
>Cancels a launchpad if no tokens were sold yet and reimburses the launchpad owner with the tokens added for sale.
<br/>

```rust
buy(id: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active, launchpad must be active.

>[!NOTE]
>Receives an amount of payment_tokens for the launchpad specified by `id` and sends back bought_amount = payment_amount / price. 
>An error is thrown if bought_amount < min_buy_amount or if alread_bought + bought_amount > max_buy_amount.
<br/>

```rust
launch(id: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active, launchpad must have ended and must not be already launched.

>[!NOTE]
>The owner of the launchpad specified by `id` receives the unsold tokens and the raised funds.
<br/>

```rust
setStateActive()
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>Sets the SC state as active.
<br/>

```rust
setStateInactive()
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>Sets the SC state as inactive.
<br/>

```rust
setPlatformAddress(platform_sc: ManagedAddress)
```
>[!IMPORTANT]
>*Requirements:* caller = owner, platform should be empty.

>[!NOTE]
>Sets the Platform SC address and retrieves the governance token id from it.

<br/>

## View functions

```rust
getState() -> State
```
>Returns the state of the SC (Active or Inactive).
<br/>

```rust
getPlatformAddress() -> ManagedAddress
```
>Returns the Platform SC address if set.
<br/>

```rust
getLaunchpad(id: u64) -> Launchpad
```
>Returns the Launchpad object associated with the `id` parameter.
<br/>

```rust
getAllLaunchpads(
    start_idx: u64,
    end_idx: u64,
    address: ManagedAddress,
    status: OptionalValue<Status>,
) -> ManagedVec<Launchpad>
```
>Returns a list with all the launchpads. The `user_bought` field of each launchpad is populated with `address`'s participation amount in the respective launchpad. 
>If `start_idx` and `end_idx` are non zero, only the launchpads in between are returned (useful for pagination). if `status` is specified, only the launchpads with that status are returned.
<br/>

```rust
getLaunchpadsCount(status: OptionalValue<Status>) -> u64
```
>Returns the count of all launchpads. If `status` is specified, the value returned represents the count of all launchpads with that status (also useful for pagination).

<br/>

## Custom types

```rust
pub enum State {
    Inactive,
    Active,
}
```

<br/>

```rust
pub enum Status {
    Pending,
    Active,
    Ended,
    Launched,
}
```

<br/>

```rust
pub struct Launchpad<M: ManagedTypeApi> {
    pub id: u64,
    pub owner: ManagedAddress<M>,
    pub description: ManagedBuffer<M>,
    pub token: TokenIdentifier<M>, // should have 18 decimals. please check in front end
    pub amount: BigUint<M>,
    pub payment_token: TokenIdentifier<M>,
    pub price: BigUint<M>, // if payment token is USDC (6 decimals), price should be x_000_000
    pub min_buy_amount: BigUint<M>,
    pub max_buy_amount: BigUint<M>,
    pub start_time: u64,
    pub end_time: u64,
    pub total_raised: BigUint<M>,
    pub total_sold: BigUint<M>,
    pub launched: bool,
    pub status: Status,
    pub user_bought: BigUint<M>,
}
```
